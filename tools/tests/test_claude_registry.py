"""The Claude session registry (config/osd/.config/osd/claude_registry.py) without GTK."""

import importlib.util
import json
import subprocess
import time
import os
import sys
import unittest
from unittest import mock

from support import OSD, SESSION_A, SESSION_B, Sandbox


def load_registry(runtime_dir):
    os.environ["XDG_RUNTIME_DIR"] = str(runtime_dir)
    spec = importlib.util.spec_from_file_location("claude_registry", OSD / "claude_registry.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    # The module reads XDG_STATE_HOME of the process running the tests, which is the real one:
    # without this, test sessions end up in the real log.
    module.LOG_PATH = os.devnull
    return module


class FakeBanners:
    def __init__(self):
        self.raised, self.dropped = [], []

    def raise_banner(self, banner):
        self.raised.append(banner["session"])

    def showing(self, session):
        return session in self.raised and session not in self.dropped

    def raise_notice(self, notice):
        self.raised.append(notice["id"])

    def drop(self, session):
        self.dropped.append(session)


class PaneStateTests(unittest.TestCase):
    def setUp(self):
        self.registry = load_registry("/tmp")

    def test_spinner_line_means_working(self):
        self.assertEqual(self.registry.pane_state("✻ Spinning… (8m 31s · ↓ 29.7k tokens)\n❯ \n"), "working")

    def test_interrupt_hint_line_means_working(self):
        self.assertEqual(self.registry.pane_state("❯ \n  enter to interrupt and send · ctrl+x then enter to queue\n"), "working")

    def test_permission_dialog_means_needs_you(self):
        self.assertEqual(self.registry.pane_state(" Do you want to proceed?\n ❯ 1. Yes\n   2. No\n"), "needs-you")

    def test_a_question_menu_means_needs_you_not_the_idle_prompt(self):
        question = " ☐ Layout\n\n Which guide layout do you want?\n\n ❯ 1. A: one step, one box\n   2. B: checklist + box\n   3. Type something.\n\n Enter to select · ↑/↓ to navigate · Esc to cancel\n"
        self.assertEqual(self.registry.pane_state(question), "needs-you")

    def test_a_numbered_line_typed_at_the_prompt_alone_is_not_a_menu(self):
        self.assertEqual(self.registry.pane_state("✻ Brewed for 3s\n❯ 1.\n"), "waiting")

    def test_idle_prompt_with_no_break_space_means_waiting(self):
        self.assertEqual(self.registry.pane_state("✻ Brewed for 23s · done\n❯ \n  [theo@archlinux dotfiles]\n"), "waiting")

    def test_phrases_inside_conversation_text_do_not_count(self):
        prose = "- `Do you want to proceed?` on screen: needs-you.\n- esc to interrupt: working.\n❯ \n"
        self.assertEqual(self.registry.pane_state(prose), "waiting")

    def test_unrecognised_text_is_none(self):
        self.assertIsNone(self.registry.pane_state("random scrollback\n"))


class ParseSessionTests(unittest.TestCase):
    def setUp(self):
        self.registry = load_registry("/tmp")

    def test_accepts_a_full_report(self):
        report = self.registry.parse_session(json.dumps({
            "session": SESSION_A, "state": "working", "project": "p", "tmux_pane": "%4",
            "kitty_pid": "42", "kitty_listen": "unix:@kitty-42"}))
        self.assertEqual(report["tmux_pane"], "%4")
        self.assertFalse(report["sticky"])

    def test_state_is_optional(self):
        report = self.registry.parse_session(json.dumps({"session": SESSION_A, "project": "p", "message": "info"}))
        self.assertNotIn("state", report)

    def test_rejects_injection_in_focus_fields(self):
        for bad in ({"tmux_pane": "%2; rm -rf /"}, {"kitty_pid": '12") os.exit'}, {"kitty_listen": "unix:@a b"}):
            with self.subTest(bad=bad):
                with self.assertRaises(ValueError):
                    self.registry.parse_session(json.dumps({"session": SESSION_A, "project": "p", **bad}))

    def test_rejects_unknown_state_and_missing_project(self):
        with self.assertRaises(ValueError):
            self.registry.parse_session(json.dumps({"session": SESSION_A, "state": "bogus", "project": "p"}))
        with self.assertRaises(ValueError):
            self.registry.parse_session(json.dumps({"session": SESSION_A, "state": "working"}))

    def test_rejects_tmux_formats_in_text(self):
        with self.assertRaises(ValueError):
            self.registry.parse_session(json.dumps({"session": SESSION_A, "project": "p", "task": "#(id)"}))


class RegistryTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.registry = load_registry(self.runtime)
        self.hyprctl_calls = []
        self.registry.run_command = lambda command, ok_exit_codes=(0,): True
        self.registry.hyprctl_eval = lambda expression: self.hyprctl_calls.append(expression) or True
        self.registry.resolve_target = lambda target: target
        self.registry.claude_workspace_active = lambda: False
        self.registry.in_meeting = lambda: False  # never the real khal in tests
        self.registry.RECENT_SESSIONS_PATH = str(self.state / "recent.json")
        self.registry.claude_panes = lambda: set()
        self.registry.live_session_ids = lambda: set()
        self.registry.tmux_output = lambda arguments: ""
        self.banners = FakeBanners()
        self.sessions = self.registry.SessionRegistry(self.banners, lambda seconds, callback: None)

    def report(self, session, **fields):
        self.sessions.update(self.registry.parse_session(json.dumps({"session": session, "project": "p", **fields})))

    def snapshot(self):
        return json.load(open(self.registry.SESSIONS_SNAPSHOT_PATH))["sessions"]

    def test_the_daemon_writes_its_messages_to_the_shared_log(self):
        self.registry.LOG_PATH = str(self.state / "claude-sessions/log")
        self.registry.log("session aaaaaaaa (runway) is gone, dropping it")
        line = (self.state / "claude-sessions/log").read_text().splitlines()[-1]
        self.assertRegex(line, r"[+-]\d\d:\d\d INFO daemon: session aaaaaaaa \(runway\) is gone, dropping it$")

    def test_permission_request_reads_the_dialog_above_the_question(self):
        boxed = ("✻ Thinking…\n╭──────────────────────────╮\n│ Bash command             │\n│                          │\n"
                 "│   npm test -- --watch=false │\n│   Run the test suite     │\n│                          │\n"
                 "│ Do you want to proceed?  │\n│ ❯ 1. Yes                 │\n│   2. No                  │\n╰──────────╯\n")
        self.assertEqual(self.registry.permission_request(boxed), "Bash command · npm test -- --watch=false")
        self.assertEqual(self.registry.pane_state(boxed), "needs-you")
        plain = "done\n Edit file\n src/app.ts\n Do you want to proceed?\n ❯ 1. Yes\n"
        self.assertEqual(self.registry.permission_request(plain), "done · Edit file")
        self.assertEqual(self.registry.permission_request("❯ \n"), "")
        self.assertNotIn("#{", self.registry.permission_request(" Bash\n echo #{pane}\n Do you want to proceed?\n"))

    def test_a_waiting_session_carries_what_it_asks_until_it_moves_on(self):
        self.report(SESSION_A, state="needs-you", message="Claude needs your permission to use Bash", sticky=True)
        self.assertEqual(self.snapshot()[0]["request"], "Claude needs your permission to use Bash")
        self.registry.claude_panes = lambda: {"%5"}
        self.registry.tmux_output = lambda arguments: " Bash command\n   npm test\n Do you want to proceed?\n ❯ 1. Yes\n"
        self.report(SESSION_A, tmux_pane="%5")
        self.sessions.prune_dead()
        self.assertEqual(self.snapshot()[0]["detail"], "Bash command · npm test")
        self.report(SESSION_A, state="working")
        self.assertEqual((self.snapshot()[0]["request"], self.snapshot()[0]["detail"]), ("", ""))

    def test_only_a_timed_event_under_way_is_a_meeting(self):
        now = time.mktime((2026, 10, 19, 10, 30, 0, 0, 0, -1))
        meeting = "19/10/2026 10:00|19/10/2026 11:00|Standup\n"
        self.assertTrue(self.registry.events_now("Today, 19/10/2026\n" + meeting, now))
        self.assertFalse(self.registry.events_now("19/10/2026|20/10/2026|Holiday\n", now))
        self.assertFalse(self.registry.events_now("19/10/2026 09:00|19/10/2026 10:30|Done\n", now))
        self.assertTrue(self.registry.events_now("19/10/2026 10:31|19/10/2026 11:00|Starts now\n", now))

    def test_banners_wait_for_the_end_of_a_meeting(self):
        meeting = {"now": True}
        self.registry.in_meeting = lambda: meeting["now"]
        self.report(SESSION_A, state="needs-you", message="Claude needs your permission to use Bash", sticky=True)
        self.assertEqual(self.banners.raised, [])
        meeting["now"] = False
        self.sessions.prune_dead()
        self.assertEqual(self.banners.raised, [SESSION_A])

    def test_sessions_are_remembered_across_reboots(self):
        self.report(SESSION_A, state="waiting", cwd="/r/runway", project="runway")
        self.report(SESSION_A, state="gone")
        recent = json.load(open(self.state / "recent.json"))
        self.assertEqual((recent[SESSION_A]["project"], recent[SESSION_A]["cwd"]), ("runway", "/r/runway"))

    def git(self, *arguments):
        subprocess.run(["git", "-C", str(self.root / "repo"), "-c", "user.email=t@t", "-c", "user.name=t", *arguments],
                       check=True, capture_output=True)

    def test_a_finished_turn_reports_its_commits_and_uncommitted_changes(self):
        repo = self.root / "repo"
        repo.mkdir()
        self.git("init", "-q")
        (repo / "a.txt").write_text("one\n")
        self.git("add", "a.txt")
        self.git("commit", "-qm", "start")
        self.report(SESSION_A, state="waiting", cwd=str(repo), project="repo")
        self.report(SESSION_A, state="working")
        (repo / "b.txt").write_text("two\n")
        self.git("add", "b.txt")
        self.git("commit", "-qm", "work")
        (repo / "a.txt").write_text("one\nmore\nlines\n")
        self.report(SESSION_A, state="waiting", message="Finished, waiting for you")
        self.assertEqual(self.snapshot()[0]["outcome"], "1 commit · 1 file changed, +2 −0 uncommitted")
        self.assertEqual(self.registry.turn_outcome(str(self.root), ""), "")  # not a repository

    def test_a_banner_left_up_while_the_session_works_is_removed(self):
        self.report(SESSION_A, state="needs-you", message="Claude needs your permission to use Bash", sticky=True)
        self.sessions.sessions[SESSION_A]["state"] = "working"  # the report that should have removed it was missed
        self.sessions.prune_dead()
        self.assertIn(SESSION_A, self.banners.dropped)

    def test_hyprctl_gets_the_running_hyprland_when_its_id_is_stale(self):
        self.registry.RUNTIME_DIR = str(self.runtime)
        folder = self.runtime / "hypr" / "running_instance"
        folder.mkdir(parents=True)
        (folder / ".socket.sock").write_text("")
        os.environ["HYPRLAND_INSTANCE_SIGNATURE"] = "gone"
        self.addCleanup(os.environ.pop, "HYPRLAND_INSTANCE_SIGNATURE", None)
        self.registry.refresh_hyprland_instance()
        self.assertEqual(os.environ["HYPRLAND_INSTANCE_SIGNATURE"], "running_instance")

    def run_setup(self, on_cockpit=False, quiet=False):
        self.registry.RUNS_DIR = str(self.root)
        commands, windows = [], []

        class Window:
            def __init__(self, command):
                self.command, self.terminated = command, False

            def poll(self):
                return 0 if self.terminated else None

            def terminate(self):
                self.terminated = True

        def run(command, ok_exit_codes=(0,)):
            commands.append(command)
            return True

        self.registry.run_command = run
        self.registry.claude_workspace_active = lambda: on_cockpit
        self.registry.quiet = lambda: quiet
        self.registry.tmux_output = lambda arguments: ""
        original = self.registry.subprocess.Popen
        self.addCleanup(setattr, self.registry.subprocess, "Popen", original)
        self.registry.subprocess.Popen = lambda command, **options: windows.append(Window(command)) or windows[-1]
        return self.registry.RunRequests(self.banners, lambda seconds, callback: None), commands, windows

    def test_a_request_starts_in_its_own_tmux_session_and_one_at_a_time(self):
        runs, commands, _ = self.run_setup()
        with self.assertRaises(ValueError):
            runs.request("../../etc")
        runs.request("aaaaaaaaaaaa")
        runs.request("bbbbbbbbbbbb")
        started = [command for command in commands if command[:2] == ["tmux", "new-session"]]
        self.assertEqual([command[4] for command in started], ["claude-run-aaaaaaaaaaaa"])
        self.assertIn(["claude-sessions", "approval-pane", "aaaaaaaaaaaa"], commands)
        runs.phase_changed("aaaaaaaaaaaa", "done")
        self.assertIn(["claude-sessions", "approval-pane", "close"], commands)
        self.assertEqual(runs.current, "bbbbbbbbbbbb")

    def test_the_window_opens_while_it_needs_theo_off_the_cockpit(self):
        runs, _, windows = self.run_setup()
        runs.request("aaaaaaaaaaaa")
        self.assertEqual(windows[0].command[-1], "=claude-run-aaaaaaaaaaaa")

    def test_the_window_stays_open_while_the_command_runs(self):
        runs, _, windows = self.run_setup()
        runs.request("aaaaaaaaaaaa")
        runs.phase_changed("aaaaaaaaaaaa", "running")
        runs.tick()
        self.assertEqual(len(windows), 1)
        self.assertFalse(windows[0].terminated)

    def test_a_window_closed_during_the_run_is_not_reopened_until_the_last_question(self):
        runs, _, windows = self.run_setup()
        runs.request("aaaaaaaaaaaa")
        runs.phase_changed("aaaaaaaaaaaa", "running")
        windows[0].terminate()
        runs.tick()
        runs.tick()
        self.assertEqual(len(windows), 1)
        runs.phase_changed("aaaaaaaaaaaa", "asking")
        self.assertEqual(len(windows), 2)

    def test_the_running_window_closes_when_a_game_starts(self):
        runs, _, windows = self.run_setup()
        runs.request("aaaaaaaaaaaa")
        runs.phase_changed("aaaaaaaaaaaa", "running")
        self.registry.quiet = lambda: True
        runs.tick()
        self.assertTrue(windows[0].terminated)

    def test_no_window_on_the_cockpit_or_while_gaming(self):
        for on_cockpit, quiet in ((True, False), (False, True)):
            runs, _, windows = self.run_setup(on_cockpit=on_cockpit, quiet=quiet)
            runs.request("aaaaaaaaaaaa")
            self.assertEqual(windows, [])

    def test_gaming_means_game_mode_or_a_game_window(self):
        original = self.registry.subprocess.run
        self.addCleanup(setattr, self.registry.subprocess, "run", original)
        for submap, window_class, expected in (("gamemode", "kitty", True), ("default", "steam_app_1245620", True),
                                               ("default", "zen", False)):
            outputs = {"submap": submap, "-j": json.dumps({"class": window_class, "fullscreen": 2})}
            self.registry.subprocess.run = lambda command, **options: type("R", (), {"stdout": outputs[command[1]], "returncode": 0})()
            self.registry.gaming_cache.update(checked=0.0)
            self.assertEqual(self.registry.gaming(), expected, (submap, window_class))

    def test_notices_are_validated_like_session_text(self):
        notice = self.registry.parse_notice(json.dumps({"id": "claude-usage-80", "title": "Claude · This week at 82%", "message": "Resets Sat"}))
        self.assertEqual(notice["id"], "claude-usage-80")
        for bad in ("nope", "[]", json.dumps({"id": "A B", "title": "t", "message": "m"}),
                    json.dumps({"id": "x", "title": "", "message": "m"}), json.dumps({"id": "x", "title": "#{pane}", "message": "m"})):
            with self.assertRaises(ValueError):
                self.registry.parse_notice(bad)

    def test_a_parallel_clone_is_named_after_its_repository(self):
        clones = self.registry.CLONES_DIR
        self.assertEqual(self.registry.parallel_clone_repository(os.path.join(clones, "opusline", "feat-x")), "opusline")
        self.assertEqual(self.registry.parallel_clone_repository(os.path.join(clones, "opusline", "pr-12", "apps")), "opusline")
        self.assertEqual(self.registry.parallel_clone_repository("/home/theo/IdeaProjects/opusline"), "")
        self.assertEqual(self.registry.parallel_clone_repository(clones), "")

    def test_the_project_is_the_starting_directorys_and_stays(self):
        self.registry.project_of = lambda cwd, fallback: {"/r/opusline": "opusline", "/r/opusline/apps/api": "opusline"}.get(cwd, fallback)
        self.report(SESSION_A, state="waiting", cwd="/r/opusline", project="opusline")
        self.report(SESSION_A, state="working", cwd="/r/opusline/apps/api", project="api")
        row = self.snapshot()[0]
        self.assertEqual((row["project"], row["cwd"]), ("opusline", "/r/opusline"))
        self.assertIn("started", row)

    def test_new_session_tags_its_window_and_publishes(self):
        self.report(SESSION_A, state="waiting", kitty_pid=str(os.getpid()))
        self.assertEqual(self.hyprctl_calls, [f'CLAUDE_WINDOW_STATE({os.getpid()}, "waiting")'])
        self.assertEqual([row["session"] for row in self.snapshot()], [SESSION_A])

    def test_unchanged_state_report_costs_nothing(self):
        self.report(SESSION_A, state="working", kitty_pid="1")
        self.hyprctl_calls.clear()
        self.report(SESSION_A, state="working", kitty_pid="1")
        self.assertEqual(self.hyprctl_calls, [])

    def test_message_raises_banner_and_its_absence_drops_it(self):
        self.report(SESSION_A, state="needs-you", message="perm", sticky=True)
        self.report(SESSION_A, state="working")
        self.assertEqual(self.banners.raised, [SESSION_A])
        self.assertIn(SESSION_A, self.banners.dropped)

    def test_no_banner_while_the_claude_workspace_is_active(self):
        self.registry.claude_workspace_active = lambda: True
        self.report(SESSION_A, state="needs-you", message="perm", sticky=True)
        self.assertEqual(self.banners.raised, [])

    def test_stateless_report_keeps_the_state(self):
        self.report(SESSION_A, state="working")
        self.report(SESSION_A, message="info")
        self.assertEqual(self.sessions.sessions[SESSION_A]["state"], "working")

    def test_new_id_in_the_same_pane_replaces_the_old_one(self):
        self.report(SESSION_A, state="needs-you", tmux_pane="%5", message="perm", sticky=True)
        self.report(SESSION_B, state="working", tmux_pane="%5")
        self.assertEqual(sorted(self.sessions.sessions), [SESSION_B])
        self.assertIn(SESSION_A, self.banners.dropped)

    def test_snapshot_orders_by_urgency(self):
        self.report(SESSION_A, state="working")
        self.report(SESSION_B, state="needs-you")
        self.assertEqual([row["session"] for row in self.snapshot()], [SESSION_B, SESSION_A])

    def test_a_running_id_is_alive_and_an_older_session_falls_back_to_its_pane(self):
        self.assertFalse(self.sessions.is_dead(SESSION_A, {"tmux_pane": "%12"}, {SESSION_A}, set()))
        # A pre-2.1.300 session (no id on a command line) next to a newer one keeps its pane rule.
        self.assertFalse(self.sessions.is_dead(SESSION_B, {"tmux_pane": "%12"}, {SESSION_A}, {"%12"}))
        self.assertTrue(self.sessions.is_dead(SESSION_B, {"tmux_pane": "%12"}, {SESSION_A}, {"%3"}))

    def test_without_ids_the_pane_decides_then_the_terminal(self):
        live = {"%3"}
        self.assertFalse(self.sessions.is_dead(SESSION_A, {"tmux_pane": "%3", "kitty_pid": "999999"}, set(), live))
        self.assertTrue(self.sessions.is_dead(SESSION_A, {"tmux_pane": "%12"}, set(), live))
        self.assertTrue(self.sessions.is_dead(SESSION_A, {"kitty_pid": "999999"}, set(), live))
        self.assertFalse(self.sessions.is_dead(SESSION_A, {"kitty_pid": str(os.getpid())}, set(), live))

    def test_the_session_tree_of_claude_2_1_300_is_read_right(self):
        # Launcher, daemon (same environment, outlives the launcher), pty host, spare, the session
        # binary, and a shell whose command line mentions an id.
        tree = {
            "10": ("claude", ["/home/t/.local/bin/claude", "--resume", SESSION_B], {"TMUX_PANE": "%12"}),
            "11": ("claude", ["/home/t/.local/bin/claude", "daemon", "run", "--origin", "transient"], {"TMUX_PANE": "%12"}),
            "12": ("2.1.300", ["claude bg-pty-host", "--bg-pty-host", "/tmp/x.sock"], {}),
            "13": ("2.1.300", ["claude bg-spare", "--bg-spare", "/tmp/y.sock"], {}),
            "14": ("2.1.300", ["/home/t/.local/share/claude/versions/2.1.300", "--session-id", SESSION_A, "--fork-session"], {}),
            "15": ("bash", ["bash", "-c", f"echo --session-id {SESSION_B}"], {"TMUX_PANE": "%12"}),
        }
        registry = load_registry(self.runtime)  # setUp stubs the lookups on self.registry
        registry.list_pids = lambda: list(tree)
        registry.process_comm = lambda pid: tree[pid][0]
        registry.process_command = lambda pid: tree[pid][1]
        registry.process_environment = lambda pid: tree[pid][2]
        self.assertEqual(registry.claude_processes(), ["10", "14"])
        self.assertEqual(registry.live_session_ids(), {SESSION_A})
        self.assertEqual(registry.claude_panes(), {"%12"})  # the launcher's pane, not the daemon's
        del tree["10"]
        self.assertEqual(registry.claude_panes(), set())

    def test_prune_drops_sessions_whose_pane_lost_its_claude(self):
        self.report(SESSION_A, state="working", tmux_pane="%7")
        self.registry.claude_panes = lambda: set()
        self.registry.live_session_ids = lambda: set()
        self.sessions.prune_dead()
        self.assertEqual(self.sessions.sessions, {})

    def test_reconcile_flags_a_session_whose_claude_updated_itself(self):
        panes = {"%5": "❯ \n  ✓ Update installed · Restart to update\n"}
        self.registry.claude_panes = lambda: {"%5"}
        self.registry.tmux_output = lambda arguments: panes.get(arguments[3], "") if arguments[0] == "capture-pane" else ""
        self.report(SESSION_A, state="waiting", tmux_pane="%5")
        self.sessions.prune_dead()
        self.assertTrue(self.sessions.sessions[SESSION_A]["update"])
        self.assertEqual([row["update"] for row in self.snapshot()], [True])
        panes["%5"] = "❯ \n"
        self.sessions.prune_dead()
        self.assertFalse(self.sessions.sessions[SESSION_A]["update"])

    def test_reconcile_follows_the_pane_in_both_directions(self):
        panes = {"%5": "✻ Flowing… (11s)\n❯ \n", "%6": " Do you want to proceed?\n ❯ 1. Yes\n", "%7": "done\n❯ \n"}
        self.registry.claude_panes = lambda: set(panes)
        self.registry.tmux_output = lambda arguments: panes.get(arguments[3], "") if arguments[0] == "capture-pane" else ""
        for session, pane, state in ((SESSION_A, "%5", "needs-you"), (SESSION_B, "%6", "working"), ("cccccccc-0000-4000-8000-000000000000", "%7", "needs-you")):
            self.report(session, state=state, tmux_pane=pane)
        self.sessions.prune_dead()
        states = {session[:8]: entry["state"] for session, entry in self.sessions.sessions.items()}
        self.assertEqual(states, {"aaaaaaaa": "working", "bbbbbbbb": "needs-you", "cccccccc": "waiting"})

    def test_clear_forgets_everything_and_untags(self):
        self.report(SESSION_A, state="working", kitty_pid="1")
        self.hyprctl_calls.clear()
        self.sessions.clear()
        self.assertEqual(self.sessions.sessions, {})
        self.assertEqual(self.hyprctl_calls, ['CLAUDE_WINDOW_STATE(1, nil)'])

    def test_snapshot_survives_a_restart_and_skips_bad_rows(self):
        json.dump({"sessions": [
            {"session": SESSION_A, "state": "working", "project": "p", "since": 1, "tmux_pane": "%2"},
            {"session": SESSION_B, "state": "nope", "project": "p", "since": 1},
        ]}, open(self.registry.SESSIONS_SNAPSHOT_PATH, "w"))
        self.registry.claude_panes = lambda: {"%2"}
        restored = self.registry.SessionRegistry(FakeBanners(), lambda seconds, callback: None)
        self.assertEqual(list(restored.sessions), [SESSION_A])


class ResolveTargetTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.registry = load_registry(self.runtime)
        self.registry.tmux_output = lambda arguments: "S\n" if arguments[0] == "display-message" else "c1\n"

    def test_attached_client_names_the_live_window(self):
        self.registry.process_environment = lambda pid: {"KITTY_PID": str(os.getpid()), "KITTY_WINDOW_ID": "1"}
        resolved = self.registry.resolve_target({"tmux_pane": "%3", "kitty_pid": "999998"})
        self.assertEqual(resolved["kitty_pid"], str(os.getpid()))

    def test_dead_pids_are_dropped_not_trusted(self):
        self.registry.process_environment = lambda pid: {"KITTY_PID": "999999"}
        self.assertEqual(self.registry.resolve_target({"tmux_pane": "%3", "kitty_pid": "999998"}), {"tmux_pane": "%3"})

    def test_no_client_keeps_the_hook_values(self):
        self.registry.tmux_output = lambda arguments: "S\n" if arguments[0] == "display-message" else ""
        target = {"tmux_pane": "%9", "kitty_pid": str(os.getpid())}
        self.assertEqual(self.registry.resolve_target(target), target)


if __name__ == "__main__":
    unittest.main()
