#!/usr/bin/env python3
"""The Claude session registry behind the OSD daemon (osd.py imports this; no GTK here).

Every live Claude Code session, keyed by session id, fed by `session <json>` reports from
~/.claude/hooks/session-state.sh and mirrored to:
  - $XDG_RUNTIME_DIR/claude-sessions.json for the waybar chip (RTMIN+16) and the
    claude-sessions manager;
  - Hyprland, through `hyprctl eval CLAUDE_WINDOW_STATE(pid, state)` (apps/claude.lua), which
    colours the session's terminal border by state;
  - the OSD banners (the `banners` object given at construction).

A report carries session, project, usually a state (working | waiting | needs-you | gone;
absent for an informational event that leaves the state as it is), and optionally cwd, task,
message, sticky and the focus target of the session's window: kitty_pid, kitty_window,
kitty_listen, tmux_pane. The kitty fields are only a fallback: claude-auto-retry keeps a
session alive in tmux when its kitty window closes, so the registry asks tmux which client is
attached to the pane and reads that client's kitty identity instead.

Hooks run inside greywall with no Hyprland or tmux socket, so every tmux and Hyprland call
happens here. Hooks also mislead or go missing (a permission notification fires on a dialog
that auto mode then approves; a Stop fired during a daemon restart is lost), so every few
seconds the pane's last lines are read and win: `pane_state` recognises Claude's permission
dialog, spinner and idle prompt.

    python3 claude_registry.py classify < pane-text   prints that state, for claude-sessions doctor
"""
import json
import os
import re
import subprocess
import sys
import time

RUNTIME_DIR = os.environ.get("XDG_RUNTIME_DIR", "/tmp")
SESSIONS_SNAPSHOT_PATH = os.path.join(RUNTIME_DIR, "claude-sessions.json")
WAYBAR_SESSIONS_SIGNAL = "-RTMIN+16"
SESSION_PRUNE_INTERVAL_S = 5
# Sessions seen recently, kept across reboots for the sidebar's "bring them back" (B): id ->
# project, starting directory, last time seen. Shutdown closes sessions like an exit does, so
# nothing is removed on "gone"; entries age out after RECENT_SESSIONS_DAYS instead.
RECENT_SESSIONS_PATH = os.path.join(os.environ.get("XDG_STATE_HOME", os.path.expanduser("~/.local/state")),
                                    "claude-sessions", "recent-sessions.json")
RECENT_SESSIONS_DAYS = 7
RECENT_SESSIONS_WRITE_S = 60
COMMAND_TIMEOUT_S = 3
STATE_URGENCY = {"working": 0, "waiting": 1, "needs-you": 2}
TARGET_FIELDS = ("kitty_pid", "kitty_window", "kitty_listen", "tmux_pane")
CLAUDE_WORKSPACE = 6  # the cockpit (hypr/apps/claude.lua): its sidebar shows every state, banners are noise there

# What Claude Code prints at the bottom of its pane: while busy, while a permission dialog is open,
# and the input prompt line it shows when idle (also shown while busy, so busy wins over it).
# Busy: the spinner line, "✻ Spinning… (8m 31s · ↓ 29.7k tokens)", or the interrupt hint as a whole
# line. The bare phrases are not enough: they can scroll by inside ordinary conversation text.
BUSY_PATTERNS = (
    re.compile(r"^\s*\S{0,2}\s*[A-Z][a-z]+… \(\d+[smh]", re.MULTILINE),
    re.compile(r"^\s*(?:esc|enter) to interrupt(?: and send)?(?:\s*·.*)?$", re.MULTILINE),
)
# The permission dialog: its question alone on a line, then its first option within a few lines.
PERMISSION_DIALOG = re.compile(r"^[\s│]*Do you want to proceed\?[\s│]*$(?:\n.*){0,3}?\n[\s│❯]*1\.\s*Yes", re.MULTILINE)
PROMPT_LINE = re.compile(r"^\s*❯[  ]", re.MULTILINE)  # Claude pads its prompt with a no-break space
PANE_TAIL_LINES = 20

TMUX_PANE_PATTERN = re.compile(r"%[0-9]{1,10}")
KITTY_LISTEN_PATTERN = re.compile(r"unix:[@/][A-Za-z0-9_./@-]{1,200}")
ASCII_NUMBER_PATTERN = re.compile(r"[0-9]{1,10}")
# Claude's own footer once it has updated itself; the running process keeps the old version until
# restarted, which the manager's r does in place.
UPDATE_READY = re.compile(r"Update installed\s*·\s*Restart to update")
SESSION_ID_PATTERN = re.compile(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
MAX_TEXT_LENGTH = 300


LOG_PATH = os.path.join(os.environ.get("XDG_STATE_HOME", os.path.expanduser("~/.local/state")), "claude-sessions", "log")
LOG_MAX_BYTES = 1_048_576


def log(message: str):
    """To the journal (osd.service), and to the log the session manager shares with the sidebar."""
    print(f"osd: {message}", file=sys.stderr)
    try:
        os.makedirs(os.path.dirname(LOG_PATH), exist_ok=True)
        if os.path.exists(LOG_PATH) and os.path.getsize(LOG_PATH) > LOG_MAX_BYTES:
            os.replace(LOG_PATH, LOG_PATH + ".1")
        stamp = time.strftime("%Y-%m-%dT%H:%M:%S%z")
        stamp = stamp[:-2] + ":" + stamp[-2:]
        with open(LOG_PATH, "a") as shared:
            shared.write(f"{stamp} INFO daemon: {' '.join(message.split())[:300]}\n")
    except OSError:
        pass


def validate_focus_target(fields: dict):
    """The focus fields end up in shell arguments and a Lua string, so they are strict."""
    for key in ("kitty_pid", "kitty_window"):
        if key in fields and not ASCII_NUMBER_PATTERN.fullmatch(str(fields[key])):
            raise ValueError(f"{key} is not a number")
    if "kitty_listen" in fields and not KITTY_LISTEN_PATTERN.fullmatch(str(fields["kitty_listen"])):
        raise ValueError("kitty_listen is not a unix socket address")
    if "tmux_pane" in fields and not TMUX_PANE_PATTERN.fullmatch(str(fields["tmux_pane"])):
        raise ValueError("tmux_pane is not a pane id")


def validate_text(key: str, value: str):
    """Session text reaches tmux and the session manager, where #( and #{ are live tmux formats."""
    if len(value) > MAX_TEXT_LENGTH:
        raise ValueError(f"session.{key} is too long")
    if any(ord(character) < 32 or ord(character) == 127 for character in value):
        raise ValueError(f"session.{key} contains control characters")
    if "#(" in value or "#{" in value:
        raise ValueError(f"session.{key} contains a tmux format sequence")


def parse_session(raw: str) -> dict:
    report = json.loads(raw)
    if not isinstance(report, dict):
        raise ValueError("session report is not an object")
    for key in ("session", "project"):
        if not isinstance(report.get(key), str) or not report[key]:
            raise ValueError(f"session.{key} missing")
    if "state" in report and report["state"] not in STATE_URGENCY and report["state"] != "gone":
        raise ValueError(f"session.state {report['state']!r} unknown")
    for key in ("cwd", "task", "message"):
        if key in report and not isinstance(report[key], str):
            raise ValueError(f"session.{key} is not a string")
    if not SESSION_ID_PATTERN.fullmatch(report["session"]):
        raise ValueError("session is not a Claude session id")
    for key in ("session", "project", "cwd", "task", "message"):
        if key in report:
            validate_text(key, report[key])
    report["sticky"] = bool(report.get("sticky", False))
    validate_focus_target(report)
    return report


NOTICE_ID = re.compile(r"[a-z][a-z0-9:-]{0,63}")


def parse_notice(raw: str) -> dict:
    """notice {"id", "title", "message"}: a banner that is not a session (the sidebar's budget
    alerts). The id names it for notice-clear; the texts follow the session text rules."""
    try:
        notice = json.loads(raw)
    except ValueError as error:
        raise ValueError(f"not JSON: {error}") from None
    if not isinstance(notice, dict):
        raise ValueError("not an object")
    if not isinstance(notice.get("id"), str) or not NOTICE_ID.fullmatch(notice["id"]):
        raise ValueError("bad id")
    for key in ("title", "message"):
        if not isinstance(notice.get(key), str) or not notice[key]:
            raise ValueError(f"bad {key}")
        validate_text(key, notice[key])
    return {"id": notice["id"], "title": notice["title"], "message": notice["message"]}


def tmux_output(arguments: list[str]):
    try:
        result = subprocess.run(["tmux", *arguments], capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    return result.stdout


def process_environment(pid: str) -> dict[str, str]:
    try:
        with open(f"/proc/{pid}/environ", "rb") as environ:
            entries = environ.read().split(b"\0")
    except OSError:
        return {}
    pairs = (entry.decode(errors="replace").partition("=") for entry in entries if entry)
    return {name: value for name, _, value in pairs}


def process_is_alive(pid: str) -> bool:
    try:
        os.kill(int(pid), 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        pass
    return True


def without_dead_kitty(target: dict) -> dict:
    """A KITTY_PID inherited from a tmux server's environment can name a long closed terminal."""
    if target.get("kitty_pid") and not process_is_alive(target["kitty_pid"]):
        return {key: value for key, value in target.items() if not key.startswith("kitty_")}
    return target


def process_command(pid: str) -> list[str]:
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as cmdline:
            return [part.decode(errors="replace") for part in cmdline.read().split(b"\0") if part]
    except OSError:
        return []


# Since Claude Code 2.1.300 a session is a tree: the launcher `claude`, under it `claude daemon
# run` (comm claude as well, the launcher's environment inherited, and it outlives the launcher),
# a `bg-pty-host`, and the session itself, a binary named after the version with
# `--session-id <id>` on its command line. Only the launcher and the session count as Claude
# processes; the daemon and the pty hosts would keep a gone session's pane "alive".
CLAUDE_VERSION_COMM = re.compile(r"\d+\.\d+\.\d+")
INFRASTRUCTURE_ARGUMENTS = {"daemon", "bg-pty-host", "bg-spare"}


def list_pids() -> list[str]:
    try:
        with os.scandir("/proc") as entries:
            return [entry.name for entry in entries if entry.name.isdigit()]
    except OSError:
        return []


def process_comm(pid: str) -> str:
    try:
        with open(f"/proc/{pid}/comm") as comm:
            return comm.read().strip()
    except OSError:
        return ""


project_cache: dict[str, str] = {}
# claude-sessions puts parallel sessions in clones at <CLONES_DIR>/<repository>/<name>.
CLONES_DIR = os.environ.get("CLAUDE_SESSIONS_CLONES", os.path.expanduser("~/IdeaProjects/.parallel"))


def parallel_clone_repository(cwd: str) -> str:
    """The repository a claude-sessions clone was made from, or empty when cwd is not in one."""
    relative = os.path.relpath(cwd, CLONES_DIR)
    parts = relative.split(os.sep)
    return parts[0] if len(parts) >= 2 and not relative.startswith("..") else ""


def project_of(cwd: str, fallback: str) -> str:
    """The repository a directory belongs to, named after its main checkout: a session that moves
    into a subfolder (opusline/api) or runs in a worktree keeps the project's own name. Outside a
    repository, the directory's name; without a directory, what the hook reported."""
    if not cwd:
        return fallback
    clone_repository = parallel_clone_repository(cwd)
    if clone_repository:
        return clone_repository
    if cwd not in project_cache:
        try:
            result = subprocess.run(["git", "-C", cwd, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                    capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
            common = result.stdout.strip() if result.returncode == 0 else ""
        except (OSError, subprocess.TimeoutExpired):
            common = ""
        if common.endswith("/.git"):
            project_cache[cwd] = os.path.basename(os.path.dirname(common))
        elif common:
            project_cache[cwd] = os.path.basename(common).removesuffix(".git")
        else:
            project_cache[cwd] = os.path.basename(cwd.rstrip("/")) or fallback
    return project_cache[cwd]


MEETING_CHECK_S = 60
KHAL_EVENT = re.compile(r"^(\d{2})/(\d{2})/(\d{4}) (\d{2}):(\d{2})\|(\d{2})/(\d{2})/(\d{4}) (\d{2}):(\d{2})\|")
meeting_cache = {"checked": 0.0, "now": False}


def events_now(khal_output: str, now: float) -> bool:
    """A timed khal event under way now (or starting within the minute). All-day events, which
    khal lists with a bare date, do not count: a holiday is not a meeting. khal prints dd/mm/yyyy
    (~/.config/khal/config), as waybar's calendar.sh expects too."""
    for line in khal_output.splitlines():
        match = KHAL_EVENT.match(line)
        if not match:
            continue
        day, month, year, hour, minute, end_day, end_month, end_year, end_hour, end_minute = map(int, match.groups())
        start = time.mktime((year, month, day, hour, minute, 0, 0, 0, -1))
        end = time.mktime((end_year, end_month, end_day, end_hour, end_minute, 0, 0, 0, -1))
        if start <= now + 60 and end > now:
            return True
    return False


def in_meeting() -> bool:
    """Whether the calendar shows a meeting now; khal is asked at most once a minute."""
    if time.time() - meeting_cache["checked"] < MEETING_CHECK_S:
        return meeting_cache["now"]
    try:
        result = subprocess.run(["khal", "list", "now", "1m", "--format", "{start}|{end}|{title}"],
                                capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
        now = events_now(result.stdout, time.time()) if result.returncode == 0 else False
    except (OSError, subprocess.TimeoutExpired):
        now = False
    if now != meeting_cache["now"]:
        log("meeting under way: Claude banners held until it ends" if now else "no meeting: banners shown again")
    meeting_cache.update(checked=time.time(), now=now)
    return now


FINISHED_MESSAGE = "Finished, waiting for you"  # the hook's Stop message (session-state.sh)


def git_output(cwd: str, *arguments: str) -> str:
    try:
        result = subprocess.run(["git", "-C", cwd, *arguments], capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired):
        return ""
    return result.stdout.strip() if result.returncode == 0 else ""


def turn_outcome(cwd: str, start_head: str) -> str:
    """What a turn left in the repository: commits since start_head, and the uncommitted changes.
    "no changes" when neither; empty outside a git repository."""
    if not cwd or not git_output(cwd, "rev-parse", "--git-dir"):
        return ""
    parts = []
    if start_head:
        commits = git_output(cwd, "rev-list", "--count", f"{start_head}..HEAD")
        if commits.isdigit() and int(commits):
            parts.append(f"{commits} commit{'s' if commits != '1' else ''}")
    numbers = re.findall(r"(\d+) (file|insertion|deletion)", git_output(cwd, "diff", "--shortstat", "HEAD"))
    counts = {kind: int(number) for number, kind in numbers}
    if counts.get("file"):
        parts.append(f"{counts['file']} file{'s' if counts['file'] != 1 else ''} changed, "
                     f"+{counts.get('insertion', 0)} −{counts.get('deletion', 0)} uncommitted")
    return " · ".join(parts) or "no changes"


def claude_processes() -> list[str]:
    """Pids of the launchers and the session binaries running right now."""
    found = []
    for pid in list_pids():
        name = process_comm(pid)
        if name != "claude" and not CLAUDE_VERSION_COMM.fullmatch(name):
            continue
        command = process_command(pid)
        # The pty host's argv[0] is the two words "claude bg-pty-host"; the daemon's role is argv[1].
        leading_words = command[0].split()[1:] + command[1:2] if command else []
        if INFRASTRUCTURE_ARGUMENTS & set(leading_words):
            continue
        found.append(pid)
    return found


def live_session_ids() -> set[str]:
    """The session ids on the command lines of the running session binaries."""
    ids = set()
    for pid in claude_processes():
        command = process_command(pid)
        for index, argument in enumerate(command[:-1]):
            if argument == "--session-id" and SESSION_ID_PATTERN.fullmatch(command[index + 1]):
                ids.add(command[index + 1])
    return ids


def claude_panes() -> set[str]:
    """The tmux panes in which a Claude process is running right now, read from their environment:
    the rule for sessions whose id is not on a command line (older Claude versions)."""
    panes = set()
    for pid in claude_processes():
        pane = process_environment(pid).get("TMUX_PANE")
        if pane:
            panes.add(pane)
    return panes


PROCEED_LINE = re.compile(r"^[\s│]*Do you want to proceed\?[\s│]*$")
BOX_LINE = re.compile(r"^[\s│]*[╭╰─┌└━]")
REQUEST_MAX_LENGTH = 140


def permission_request(tail: str) -> str:
    """What a permission dialog asks, from the pane: the lines above "Do you want to proceed?" up
    to the dialog's top edge, such as "Bash command · npm test". Empty when there is no dialog."""
    lines = tail.splitlines()
    ask = next((index for index in range(len(lines) - 1, -1, -1) if PROCEED_LINE.match(lines[index])), None)
    if ask is None:
        return ""
    block = []
    for line in reversed(lines[max(0, ask - 10):ask]):
        if BOX_LINE.match(line):
            break
        text = line.strip().strip("│").strip()
        if text:
            block.insert(0, text)
    request = " · ".join(block[:2])
    request = "".join(character for character in request if ord(character) >= 32 and ord(character) != 127)
    return request.replace("#{", "#\u200b{").replace("#(", "#\u200b(")[:REQUEST_MAX_LENGTH]


def pane_state(tail: str):
    """needs-you on a permission dialog, working on the spinner, waiting at the idle prompt, else None."""
    if PERMISSION_DIALOG.search(tail):
        return "needs-you"
    if any(pattern.search(tail) for pattern in BUSY_PATTERNS):
        return "working"
    if PROMPT_LINE.search(tail):
        return "waiting"
    return None


def resolve_target(target: dict) -> dict:
    """The kitty window currently attached to the session's tmux pane, or the reported one.

    The attached tmux client inherited KITTY_* from the shell of the window it runs in, so
    its environment names the live window. With no client attached yet (the launcher
    attaches right after creating the session) the hook's values are the right ones."""
    tmux_pane = target.get("tmux_pane")
    if not tmux_pane:
        return without_dead_kitty(target)
    session_name = tmux_output(["display-message", "-p", "-t", tmux_pane, "#{session_name}"])
    if not session_name:
        return without_dead_kitty(target)
    client_pids = (tmux_output(["list-clients", "-t", session_name.strip(), "-F", "#{client_pid}"]) or "").split()
    for client_pid in client_pids:
        environment = process_environment(client_pid)
        if not environment.get("KITTY_PID") or not process_is_alive(environment["KITTY_PID"]):
            continue
        resolved = {"tmux_pane": tmux_pane, "kitty_pid": environment["KITTY_PID"]}
        for key, variable in (("kitty_window", "KITTY_WINDOW_ID"), ("kitty_listen", "KITTY_LISTEN_ON")):
            if environment.get(variable):
                resolved[key] = environment[variable]
        try:
            validate_focus_target(resolved)
        except ValueError as error:
            log(f"ignoring tmux client {client_pid}: {error}")
            continue
        return resolved
    return without_dead_kitty(target)


def refresh_hyprland_instance():
    """hyprctl finds Hyprland through HYPRLAND_INSTANCE_SIGNATURE. Processes that outlive a Hyprland
    restart, or start outside it (the manager's tmux server, systemd units), hold a stale or no
    value: take the running instance, the newest $XDG_RUNTIME_DIR/hypr/<id> with a socket."""
    base = os.path.join(RUNTIME_DIR, "hypr")
    current = os.environ.get("HYPRLAND_INSTANCE_SIGNATURE", "")
    if current and os.path.exists(os.path.join(base, current, ".socket.sock")):
        return
    try:
        running = [entry for entry in os.scandir(base) if os.path.exists(os.path.join(entry.path, ".socket.sock"))]
    except OSError:
        return
    if running:
        os.environ["HYPRLAND_INSTANCE_SIGNATURE"] = max(running, key=lambda entry: entry.stat().st_mtime).name


def run_command(command: list[str], ok_exit_codes=(0,)) -> bool:
    if command[0] == "hyprctl":
        refresh_hyprland_instance()
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired) as error:
        log(f"{command[0]} failed: {error}")
        return False
    if result.returncode not in ok_exit_codes:
        output = (result.stdout + result.stderr).strip()
        log(f"{' '.join(command)} exited {result.returncode}: {output}")
        return False
    return True


def hyprctl_eval(expression: str) -> bool:
    return run_command(["hyprctl", "eval", expression])


def claude_workspace_active() -> bool:
    refresh_hyprland_instance()
    try:
        result = subprocess.run(["hyprctl", "-j", "activeworkspace"], capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
        return result.returncode == 0 and json.loads(result.stdout).get("id") == CLAUDE_WORKSPACE
    except (OSError, subprocess.TimeoutExpired, ValueError, AttributeError):
        return False


def show_in_manager(session: str) -> bool:
    return run_command(["claude-sessions", "show", session])


RUNS_DIR = os.environ.get("CLAUDE_RUNS_DIR", os.path.expanduser("~/.cache/claude/runs"))
RUN_ID = re.compile(r"[0-9a-f]{12}")
RUN_APPROVE = os.path.expanduser("~/.local/bin/claude-run-approve")
MANAGER_SESSION = "claude-manager"


def launch_approval(request_id: str):
    """The approval terminal for a request (claude-run-approve): a popup over the cockpit when it
    shows, else a floating window (apps/claude.lua floats class claude-run). None if it failed."""
    refresh_hyprland_instance()
    client = (tmux_output(["list-clients", "-t", MANAGER_SESSION, "-F", "#{client_name}"]) or "").split("\n")[0].strip()
    if client and claude_workspace_active():
        command = ["tmux", "display-popup", "-c", client, "-E", "-w", "80%", "-h", "70%",
                   "-T", " Claude asks to run a command ", RUN_APPROVE, request_id]
    else:
        command = ["kitty", "--class", "claude-run", "-T", "Claude asks to run a command", RUN_APPROVE, request_id]
    try:
        return subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    except OSError as error:
        log(f"cannot open the approval terminal for {request_id}: {error}")
        return None


class RunRequests:
    """Commands Claude asks Theo to run outside the sandbox (ask-to-run). The daemon runs nothing:
    it raises a banner and opens claude-run-approve, one request at a time, the others waiting."""

    def __init__(self, banners):
        self.banners = banners
        self.queue: list[str] = []
        self.current = None  # (request id, the approval terminal's process)

    def request(self, request_id: str):
        if not RUN_ID.fullmatch(request_id):
            raise ValueError("not a request id")
        if request_id in self.queue or (self.current and self.current[0] == request_id):
            return
        try:
            with open(os.path.join(RUNS_DIR, f"{request_id}.request")) as file:
                cwd = str(json.load(file).get("cwd", ""))
            # From the directory, not the request's own label, which a requester could fake.
            project = project_of(cwd, "?") if os.path.isdir(cwd) else "?"
            validate_text("project", project)
        except (OSError, ValueError, AttributeError):
            project = "?"
        self.banners.raise_notice({"id": f"run-{request_id}", "title": f"Claude · {project}",
                                   "message": "asks you to run a command"})
        log(f"run request {request_id} from {project}")
        self.queue.append(request_id)
        self.next()

    def next(self):
        """Open the next waiting request once the current approval terminal is closed."""
        if self.current and self.current[1] is not None and self.current[1].poll() is None:
            return
        self.current = None
        while self.queue:
            request_id = self.queue.pop(0)
            if os.path.exists(os.path.join(RUNS_DIR, f"{request_id}.result")):
                continue
            self.current = (request_id, launch_approval(request_id))
            return


class SessionRegistry:
    """Every live Claude session, keyed by session id, mirrored to the snapshot file and to Hyprland.

    `banners` offers raise_banner(dict) and drop(session); `schedule_repeating(seconds, callback)`
    runs the callback on the host event loop (GLib in the daemon) as long as it returns True."""

    def __init__(self, banners, schedule_repeating):
        self.banners = banners
        self.sessions: dict[str, dict] = {}
        self.held: dict[str, dict] = {}  # session -> its latest banner, kept back during a meeting
        self.runs = RunRequests(banners)
        self.recent_written = (0.0, frozenset())  # when recent-sessions.json was written, for which ids
        self.load_snapshot()
        schedule_repeating(SESSION_PRUNE_INTERVAL_S, self.prune_dead)

    def update(self, report: dict):
        session = report["session"]
        if report.get("state") == "gone":
            self.remove(session)
            return
        entry = self.sessions.get(session)
        previous_state = entry["state"] if entry else None
        if entry is None:
            entry = {"state": report.get("state", "waiting"), "project": report["project"], "cwd": "", "task": "",
                     "since": time.time(), "started": time.time(), "target": {}, "update": False,
                     "request": "", "detail": ""}
            self.sessions[session] = entry
            changed = True
        else:
            changed = False
        for key in ("state", "task"):
            if key in report and entry[key] != report[key]:
                entry[key] = report[key]
                changed = True
        # What a session waiting on you asks: the hook's sentence ("Claude needs your permission to
        # use Bash"), and the pane's detail once the next reconcile reads the dialog. Gone with the wait.
        if entry["state"] == "needs-you" and report.get("message") and entry.get("request") != report["message"]:
            entry["request"] = report["message"]
            changed = True
        elif entry["state"] != "needs-you" and (entry.get("request") or entry.get("detail")):
            entry["request"], entry["detail"] = "", ""
            changed = True
        # The directory the session started in, and the project it belongs to: later reports carry
        # Claude's current directory, which wanders into subfolders.
        if report.get("cwd") and not entry["cwd"]:
            entry["cwd"] = report["cwd"]
            entry["project"] = project_of(report["cwd"], report["project"])
            changed = True
        previous_pid = entry["target"].get("kitty_pid")
        reported_target = {key: str(report[key]) for key in TARGET_FIELDS if key in report}
        entry["target"] = resolve_target({**entry["target"], **reported_target})
        kitty_pid = entry["target"].get("kitty_pid")
        self.drop_pane_predecessors(session, entry["target"].get("tmux_pane"))
        if previous_state != entry["state"]:
            entry["since"] = time.time()
        # A turn starts when the session starts working: note the commit, to tell what it did.
        if entry["state"] == "working" and previous_state != "working" and entry["cwd"]:
            entry["turn_head"] = git_output(entry["cwd"], "rev-parse", "HEAD")
            entry["outcome"] = ""
        if report.get("message") == FINISHED_MESSAGE and entry["cwd"]:
            entry["outcome"] = turn_outcome(entry["cwd"], entry.get("turn_head", ""))
            if entry["outcome"]:
                report = {**report, "message": f"Finished · {entry['outcome']}"}
            changed = True

        if "message" not in report:
            self.banners.drop(session)
            self.held.pop(session, None)
        elif not claude_workspace_active():
            banner = {"session": session, "title": f"Claude · {entry['project']}",
                      "message": report["message"], "sticky": report["sticky"]}
            if in_meeting():
                self.held[session] = banner
                log(f"banner held (meeting) for {session[:8]} ({entry['project']}): {report['message']}")
            else:
                self.banners.raise_banner(banner)
                log(f"banner {'sticky ' if report['sticky'] else ''}for {session[:8]} ({entry['project']}): {report['message']}")

        # The window a session moved away from (tmux session re-attached elsewhere) loses its
        # tag unless another session still lives there; the new one is tagged.
        window_changed = kitty_pid != previous_pid
        if window_changed and previous_pid:
            self.apply_window_state(previous_pid)
        if kitty_pid and (window_changed or previous_state != entry["state"]):
            self.apply_window_state(kitty_pid)
        if changed or window_changed:
            self.publish()

    def drop_pane_predecessors(self, session: str, tmux_pane):
        """A pane runs one Claude at a time, but /clear, --resume and forks give it a new session
        id without a SessionEnd for the old one: the previous ids of the same pane are gone."""
        if not tmux_pane:
            return
        for other, entry in list(self.sessions.items()):
            if other != session and entry["target"].get("tmux_pane") == tmux_pane:
                log(f"session {other} ({entry['project']}) replaced in {tmux_pane} by {session}")
                self.remove(other)

    def remove(self, session: str):
        self.held.pop(session, None)
        entry = self.sessions.pop(session, None)
        if entry is None:
            return
        self.banners.drop(session)
        kitty_pid = entry["target"].get("kitty_pid")
        if kitty_pid:
            self.apply_window_state(kitty_pid)
        self.publish()

    def window_state(self, kitty_pid: str):
        """Most urgent state among the sessions sharing one kitty window (several tmux windows)."""
        states = [entry["state"] for entry in self.sessions.values() if entry["target"].get("kitty_pid") == kitty_pid]
        if not states:
            return None
        return max(states, key=STATE_URGENCY.__getitem__)

    def apply_window_state(self, kitty_pid: str):
        state = self.window_state(kitty_pid)
        hyprctl_eval(f"CLAUDE_WINDOW_STATE({kitty_pid}, {json.dumps(state) if state else 'nil'})")

    def publish(self):
        rows = [
            {"session": session, "state": entry["state"], "project": entry["project"], "cwd": entry["cwd"],
             "task": entry["task"], "since": int(entry["since"]), "started": int(entry.get("started", entry["since"])),
             "update": entry.get("update", False), "request": entry.get("request", ""), "detail": entry.get("detail", ""),
             "outcome": entry.get("outcome", ""),
             **entry["target"]}
            for session, entry in self.sessions.items()
        ]
        rows.sort(key=lambda row: (-STATE_URGENCY[row["state"]], row["since"]))
        temporary_path = SESSIONS_SNAPSHOT_PATH + ".tmp"
        with open(temporary_path, "w") as snapshot:
            json.dump({"sessions": rows}, snapshot)
        os.replace(temporary_path, SESSIONS_SNAPSHOT_PATH)
        self.remember_sessions()
        # Exit 1 is "no waybar running", not an error.
        run_command(["pkill", WAYBAR_SESSIONS_SIGNAL, "waybar"], ok_exit_codes=(0, 1))

    def remember_sessions(self):
        """Refresh recent-sessions.json: when the set of sessions changes, else once a minute."""
        ids = frozenset(session for session, entry in self.sessions.items() if entry["cwd"])
        written_at, written_ids = self.recent_written
        if ids == written_ids and time.time() - written_at < RECENT_SESSIONS_WRITE_S:
            return
        try:
            with open(RECENT_SESSIONS_PATH) as saved:
                recent = json.load(saved)
            if not isinstance(recent, dict):
                recent = {}
        except (OSError, ValueError):
            recent = {}
        now = time.time()
        for session in ids:
            entry = self.sessions[session]
            recent[session] = {"project": entry["project"], "cwd": entry["cwd"], "last_seen": int(now)}
        oldest = now - RECENT_SESSIONS_DAYS * 86400
        recent = {session: row for session, row in recent.items()
                  if isinstance(row, dict) and row.get("last_seen", 0) >= oldest}
        try:
            os.makedirs(os.path.dirname(RECENT_SESSIONS_PATH), exist_ok=True)
            with open(RECENT_SESSIONS_PATH + ".tmp", "w") as saved:
                json.dump(recent, saved)
            os.replace(RECENT_SESSIONS_PATH + ".tmp", RECENT_SESSIONS_PATH)
        except OSError as error:
            log(f"cannot write {RECENT_SESSIONS_PATH}: {error}")
        self.recent_written = (now, ids)

    def load_snapshot(self):
        """A daemon restart keeps the sessions; the next hook event refreshes their state."""
        try:
            with open(SESSIONS_SNAPSHOT_PATH) as snapshot:
                rows = json.load(snapshot)["sessions"]
        except FileNotFoundError:
            return
        except (OSError, ValueError, KeyError, TypeError) as error:
            log(f"ignoring snapshot {SESSIONS_SNAPSHOT_PATH}: {error}")
            return
        for row in rows:
            try:
                target = {key: str(row[key]) for key in TARGET_FIELDS if row.get(key)}
                validate_focus_target(target)
                if row["state"] not in STATE_URGENCY:
                    raise ValueError(f"state {row['state']!r} unknown")
                self.sessions[row["session"]] = {
                    "state": row["state"], "project": project_of(row.get("cwd", ""), row["project"]), "cwd": row.get("cwd", ""),
                    "task": row.get("task", ""), "since": row.get("since", time.time()),
                    "started": row.get("started", row.get("since", time.time())),
                    "target": resolve_target(target), "update": bool(row.get("update", False)),
                }
            except (KeyError, TypeError, ValueError) as error:
                log(f"skipping snapshot row {row!r}: {error}")
        self.prune_dead()
        for kitty_pid in {entry["target"].get("kitty_pid") for entry in self.sessions.values()} - {None}:
            self.apply_window_state(kitty_pid)
        self.publish()

    def is_dead(self, session: str, target: dict, live_ids: set[str], live_panes: set[str]) -> bool:
        """Alive while a session binary carries the id. A session without one (started by a Claude
        older than 2.1.300, still running next to newer ones after an update) is alive while a
        Claude process runs in its pane (pane ids outlive their programs: a respawned pane keeps
        its id); a restart in the same pane is caught by drop_pane_predecessors instead. Without a
        pane the terminal decides."""
        if session in live_ids:
            return False
        tmux_pane = target.get("tmux_pane")
        if tmux_pane:
            return tmux_pane not in live_panes
        kitty_pid = target.get("kitty_pid")
        return bool(kitty_pid) and not process_is_alive(kitty_pid)

    def release_held(self):
        """After the meeting, the banners it held back, for the sessions still there."""
        if not self.held or in_meeting():
            return
        held, self.held = self.held, {}
        for session, banner in held.items():
            if session in self.sessions:
                self.banners.raise_banner(banner)

    def prune_dead(self) -> bool:
        live_ids, live_panes = live_session_ids(), claude_panes()
        for session, entry in list(self.sessions.items()):
            if self.is_dead(session, entry["target"], live_ids, live_panes):
                log(f"session {session} ({entry['project']}) is gone, dropping it")
                self.remove(session)
        self.reconcile_with_panes()
        self.release_held()
        self.drop_stale_banners()
        self.runs.next()
        return True  # keep the repeating schedule

    def drop_stale_banners(self):
        """A working session asks for nothing: its banner goes, even if the report that should have
        removed it was missed (a banner once stayed up after its prompt was answered in the session)."""
        for session, entry in self.sessions.items():
            if entry["state"] == "working" and self.banners.showing(session):
                log(f"banner for {session[:8]} ({entry['project']}) still up while it works: removed")
                self.banners.drop(session)

    def reconcile_with_panes(self):
        """The pane's last lines say what Claude is doing now; they win over stale hook states."""
        for session, entry in list(self.sessions.items()):
            tmux_pane = entry["target"].get("tmux_pane")
            if not tmux_pane:
                continue
            tail = tmux_output(["capture-pane", "-p", "-t", tmux_pane, "-S", f"-{PANE_TAIL_LINES}"]) or ""
            update_ready = bool(UPDATE_READY.search(tail))
            if update_ready != entry.get("update", False):
                entry["update"] = update_ready
                self.publish()
            observed = pane_state(tail)
            if observed and observed != entry["state"]:
                self.update({"session": session, "state": observed, "project": entry["project"], "sticky": False})
            detail = permission_request(tail) if entry["state"] == "needs-you" else ""
            if detail != entry.get("detail", ""):
                entry["detail"] = detail
                self.publish()

    def clear(self):
        for session in list(self.sessions):
            self.remove(session)

    def focus_most_urgent(self):
        """Show the most urgent session in the manager; with none, just open the manager."""
        candidates = [
            (STATE_URGENCY[entry["state"]], -entry["since"], session)
            for session, entry in self.sessions.items() if entry["target"].get("tmux_pane")
        ]
        if not candidates:
            run_command(["claude-sessions", "show"])
            return
        session = max(candidates)[2]
        show_in_manager(session)
        self.banners.drop(session)


def main(arguments: list[str]) -> int:
    if arguments == ["classify"]:
        print(pane_state(sys.stdin.read()) or "unknown")
        return 0
    if arguments[:1] == ["liveness"] and len(arguments) == 3:
        session, tmux_pane = arguments[1], arguments[2]
        if session in live_session_ids():
            print("by id")
        elif tmux_pane and tmux_pane in claude_panes():
            print("by pane")
        else:
            print("none")
        return 0
    print("usage: claude_registry.py classify < pane-text", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
