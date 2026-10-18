"""The session manager script (config/bin/.local/bin/claude-sessions) against a fake tmux."""

import json
import time
import subprocess
import unittest

from support import BIN, SESSION_A, SESSION_B, Sandbox

# The manager's panes are found by their @role tag; the fake keeps those tags in $SHIM_ROLES.
FAKE_TMUX = r'''
roles="${SHIM_ROLES:-/dev/null}"
case $1 in
  list-panes)
    if [[ $2 == -t && $3 == =claude-retry-* ]]; then echo "%4"
    elif [[ $* == *@role* ]]; then cat "$roles" 2>/dev/null
    else printf '0\n1\n'; fi ;;
  has-session) [[ $3 == "=claude-manager" ]] && exit 0; (( $(grep -c "send-keys" "$SHIM_LOG") < ${SHIM_EXIT_AFTER:-1} )) ;;
  display-message) if [[ $* == *pane_dead* ]]; then echo 0; elif [[ $* == *pane_id* ]]; then echo "%10"; else echo "claude-retry-9-9"; fi ;;
  split-window) [[ $* == *-P* ]] && echo "%1$(( $(wc -l < "$roles" 2>/dev/null || echo 0) + 1 ))" ;;
  set-option) if [[ $2 == -p && $5 == @role ]]; then echo "$4 $6" >> "$roles"; fi ;;
  capture-pane) printf '%b' "$SHIM_PANE_TEXT" ;;
esac
'''


class ManagerTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.shim("tmux", FAKE_TMUX)
        self.env["SHIM_ROLES"] = str(self.root / "roles")
        # A manager whose panes are already tagged, as ensure_manager leaves them.
        (self.root / "roles").write_text("%10 sidebar\n%11 viewer\n")
        self.shim("hyprctl", '[[ $1 == -j && $2 == activeworkspace ]] && echo "{\\"id\\": ${SHIM_WS:-2}}"; exit 0')
        self.shim("setsid", "")
        self.shim("osd-show", "")
        self.project = self.root / "proj-a"
        self.project.mkdir()
        self.snapshot = self.runtime / "claude-sessions.json"
        self.snapshot.write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "state": "waiting", "project": "proj-a", "cwd": str(self.project), "tmux_pane": "%4"}]}))
        # Session A has a conversation to resume (a transcript with a user message).
        transcripts = self.home / ".claude/projects/-proj-a"
        transcripts.mkdir(parents=True)
        (transcripts / f"{SESSION_A}.jsonl").write_text('{"type":"user","message":{"content":"hi"}}\n')

    def run_manager(self, *arguments, **env) -> subprocess.CompletedProcess:
        return subprocess.run([str(BIN / "claude-sessions"), *arguments], capture_output=True, text=True, env={**self.env, **env})

    def tmux_calls(self, verb: str) -> list[str]:
        return [call for call in self.calls("tmux") if call.startswith(f"tmux {verb}")]

    def test_view_attaches_the_session_nested_in_the_viewer(self):
        self.assertEqual(self.run_manager("view", SESSION_A).returncode, 0)
        self.assertEqual(self.tmux_calls("respawn-pane"), ["tmux respawn-pane -k -t %11 env -u TMUX tmux attach-session -t =claude-retry-9-9"])
        self.assertEqual((self.runtime / "claude-sessions.current").read_text().strip(), SESSION_A)

    def test_view_without_a_pane_fails_with_a_message(self):
        result = self.run_manager("view", "dddddddd-0000-4000-8000-000000000000")
        self.assertEqual(result.returncode, 1)
        self.assertIn("has no tmux pane", result.stderr)

    def test_label_state_colours_the_viewer_title_and_refuses_nonsense(self):
        self.assertEqual(self.run_manager("label-state", "needs-you").returncode, 0)
        self.assertIn("tmux set-option -p -t %11 @state needs-you", self.calls("tmux"))
        self.assertEqual(self.run_manager("label-state", "purple").returncode, 1)

    def test_open_focuses_the_viewer_after_viewing(self):
        self.run_manager("open", SESSION_A)
        self.assertIn("tmux select-pane -t %11", self.calls("tmux"))

    def test_plain_new_reuses_the_projects_running_session(self):
        self.run_manager("new", str(self.project))
        self.assertTrue(any("attach-session -t =claude-retry-9-9" in call for call in self.tmux_calls("respawn-pane")))
        self.assertFalse(any("fish -ic" in call for call in self.tmux_calls("respawn-pane")))

    def test_new_with_arguments_or_another_project_launches_fresh(self):
        other = self.root / "proj-b"
        other.mkdir()
        self.run_manager("new", str(self.project), "--resume")
        self.run_manager("new", str(other))
        launches = [call for call in self.tmux_calls("respawn-pane") if "fish -ic" in call]
        self.assertEqual(len(launches), 2)
        self.assertIn("CLAUDE_SESSIONS_INSIDE=1", launches[0])
        self.assertTrue(launches[0].endswith("--resume"))

    def test_new_refuses_a_directory_with_a_tmux_format(self):
        weird = self.root / "a#b"
        weird.mkdir()
        self.assertEqual(self.run_manager("new", str(weird)).returncode, 1)

    def test_exit_types_slash_exit_at_an_idle_prompt(self):
        self.run_manager("exit", SESSION_A, SHIM_PANE_TEXT="done\n❯ \n", SHIM_EXIT_AFTER="1")
        self.assertEqual(self.tmux_calls("send-keys"), ["tmux send-keys -t %4 /exit Enter"])
        self.assertEqual(self.tmux_calls("kill-session"), [])

    def test_exit_sends_two_quick_ctrl_c_while_busy(self):
        self.run_manager("exit", SESSION_A, SHIM_PANE_TEXT="✻ Flowing… (11s)\n❯ \n", SHIM_EXIT_AFTER="2")
        self.assertEqual(self.tmux_calls("send-keys"), ["tmux send-keys -t %4 C-c"] * 2)

    def test_exit_kills_the_tmux_session_as_a_last_resort(self):
        result = self.run_manager("exit", SESSION_A, SHIM_PANE_TEXT="✻ Flowing… (11s)\n", SHIM_EXIT_AFTER="99")
        self.assertEqual(self.tmux_calls("kill-session"), ["tmux kill-session -t =claude-retry-9-9"])
        self.assertIn("did not exit", result.stderr)

    def test_restart_exits_then_resumes_the_same_conversation(self):
        self.run_manager("restart", SESSION_A, SHIM_PANE_TEXT="done\n❯ \n", SHIM_EXIT_AFTER="1")
        calls = self.calls("tmux")
        exit_index = calls.index("tmux send-keys -t %4 /exit Enter")
        resume = next(index for index, call in enumerate(calls) if "claude --resume $argv" in call)
        self.assertLess(exit_index, resume)
        self.assertTrue(calls[resume].endswith(SESSION_A))

    def test_restart_with_an_effort_resumes_on_it(self):
        self.run_manager("restart", SESSION_A, "max", SHIM_PANE_TEXT="done\n❯ \n", SHIM_EXIT_AFTER="1")
        resume = next(call for call in self.calls("tmux") if "claude --resume $argv" in call)
        self.assertTrue(resume.endswith(f"{SESSION_A} --effort max"))

    def test_restart_keeps_the_effort_the_status_line_reported(self):
        cache = self.home / ".cache/claude"
        cache.mkdir(parents=True, exist_ok=True)
        (cache / f"statusline-{SESSION_A}.json").write_text('{"effort": "xhigh"}')
        self.run_manager("restart", SESSION_A, SHIM_PANE_TEXT="done\n❯ \n", SHIM_EXIT_AFTER="1")
        resume = next(call for call in self.calls("tmux") if "claude --resume $argv" in call)
        self.assertTrue(resume.endswith(f"{SESSION_A} --effort xhigh"))

    def test_restart_of_a_session_nobody_typed_in_starts_a_fresh_one(self):
        (self.root / "transcripts/-x").mkdir(parents=True)
        (self.root / "transcripts/-x" / f"{SESSION_A}.jsonl").write_text('{"type":"summary"}\n')
        self.run_manager("restart", SESSION_A, "max", SHIM_PANE_TEXT="done\n❯ \n", SHIM_EXIT_AFTER="1",
                         CLAUDE_SESSIONS_TRANSCRIPTS=str(self.root / "transcripts"))
        calls = self.calls("tmux")
        self.assertFalse(any("--resume" in call for call in calls))
        launch = next(call for call in calls if "claude $argv" in call)
        self.assertTrue(launch.endswith("--effort max"))

    def doctor_line(self, needle: str, **env) -> str:
        output = self.run_manager("doctor", **env).stdout
        return next(line for line in output.splitlines() if needle in line)

    def test_doctor_flags_a_claude_link_behind_the_newest_download(self):
        versions = self.root / "versions"
        versions.mkdir()
        for name in ("2.1.300", "2.1.302"):
            (versions / name).write_text("#!/bin/sh\n")
            (versions / name).chmod(0o755)
        link = self.root / "claude-link"
        link.symlink_to(versions / "2.1.300")
        env = {"CLAUDE_VERSIONS_DIR": str(versions), "CLAUDE_LINK": str(link)}
        self.assertTrue(self.doctor_line("2.1.302", **env).strip().startswith("FAIL"))
        link.unlink()
        link.symlink_to(versions / "2.1.302")
        self.assertTrue(self.doctor_line("newest version", **env).strip().startswith("ok"))

    def test_compact_is_typed_only_at_the_prompt(self):
        self.assertEqual(self.run_manager("compact", SESSION_A, SHIM_PANE_TEXT="done\n❯ \n").returncode, 0)
        self.assertIn("tmux send-keys -t %4 /compact Enter", self.calls("tmux"))
        busy = self.run_manager("compact", SESSION_A, SHIM_PANE_TEXT="✻ Flowing… (11s)\n")
        self.assertEqual(busy.returncode, 1)
        self.assertIn("busy", busy.stderr)
        self.assertEqual(len([call for call in self.calls("tmux") if "/compact" in call]), 1)

    DIALOG = " Bash command\n   npm test\n Do you want to proceed?\n ❯ 1. Yes\n   2. No\n"

    def test_approve_presses_enter_only_on_an_open_dialog_at_yes(self):
        self.assertEqual(self.run_manager("answer", SESSION_A, "yes", SHIM_PANE_TEXT=self.DIALOG).returncode, 0)
        self.assertIn("tmux send-keys -t %4 Enter", self.calls("tmux"))

    def test_nothing_is_sent_when_the_dialog_closed(self):
        result = self.run_manager("answer", SESSION_A, "yes", SHIM_PANE_TEXT="done\n❯ \n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("no permission dialog", result.stderr)
        self.assertEqual(self.tmux_calls("send-keys"), [])

    def test_decline_presses_escape(self):
        self.run_manager("answer", SESSION_A, "no", SHIM_PANE_TEXT=self.DIALOG)
        self.assertIn("tmux send-keys -t %4 Escape", self.calls("tmux"))

    def test_say_sends_at_the_prompt_and_queues_while_working(self):
        self.run_manager("say", SESSION_A, "also update the README", SHIM_PANE_TEXT="done\n❯ \n")
        self.assertEqual(self.tmux_calls("send-keys")[-2:], ["tmux send-keys -t %4 -l -- also update the README", "tmux send-keys -t %4 Enter"])
        self.run_manager("say", SESSION_A, "and the changelog", SHIM_PANE_TEXT="✻ Flowing… (11s)\n")
        self.assertEqual(self.tmux_calls("send-keys")[-1], "tmux send-keys -t %4 C-x Enter")

    def test_say_refuses_while_a_dialog_is_open_and_keeps_the_text_out_of_the_log(self):
        result = self.run_manager("say", SESSION_A, "the secret plan", SHIM_PANE_TEXT=self.DIALOG)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.tmux_calls("send-keys"), [])
        self.assertFalse(any("secret" in line for line in self.log_lines()))

    def test_zoom_toggles_the_viewer_pane(self):
        self.run_manager("zoom")
        self.assertIn("tmux resize-pane -Z -t %11", self.calls("tmux"))

    def test_restore_resumes_each_session_and_skips_what_cannot_be(self):
        result = self.run_manager("restore", str(self.project), SESSION_A, "/no/such/dir", SESSION_B,
                                  CLAUDE_SESSIONS_RESTORE_WAIT="0")
        self.assertEqual(result.returncode, 0, result.stderr)
        resumes = [call for call in self.tmux_calls("respawn-pane") if "claude --resume $argv" in call]
        self.assertEqual(len(resumes), 1)
        self.assertTrue(resumes[0].endswith(SESSION_A))

    def test_an_exit_on_purpose_is_remembered(self):
        self.run_manager("exit", SESSION_A, SHIM_PANE_TEXT="done\n❯ \n", SHIM_EXIT_AFTER="1")
        self.assertEqual((self.state / "claude-sessions/ended-sessions").read_text().split(), [SESSION_A])

    def test_git_opens_lazygit_on_the_repository_in_a_popup(self):
        self.assertEqual(self.run_manager("git", str(self.project)).returncode, 0)
        popup = self.tmux_calls("display-popup")
        self.assertEqual(len(popup), 1)
        self.assertIn(f"-d {self.project}", popup[0])
        self.assertTrue(popup[0].endswith("lazygit"))

    def git_repository(self):
        """self.project as a real git repository with one commit and a GitHub origin."""
        run = lambda *arguments: subprocess.run(["git", "-C", str(self.project), *arguments], check=True, capture_output=True)
        run("init", "-q", "-b", "main")
        (self.project / "README").write_text("x\n")
        run("add", "README")
        run("-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init")
        run("remote", "add", "origin", "git@github.com:o/proj-a.git")

    def test_parallel_session_gets_its_own_clone_on_its_branch(self):
        self.git_repository()
        clones = self.root / "clones"
        result = self.run_manager("parallel", str(self.project), "feat/effort-menu", CLAUDE_SESSIONS_CLONES=str(clones))
        self.assertEqual(result.returncode, 0, result.stderr)
        clone = clones / "proj-a" / "feat-effort-menu"
        branch = subprocess.run(["git", "-C", str(clone), "branch", "--show-current"], capture_output=True, text=True).stdout.strip()
        origin = subprocess.run(["git", "-C", str(clone), "remote", "get-url", "origin"], capture_output=True, text=True).stdout.strip()
        self.assertEqual((branch, origin), ("feat/effort-menu", "git@github.com:o/proj-a.git"))
        self.assertTrue(any(f"-c {clone} " in call and "claude $argv" in call for call in self.tmux_calls("respawn-pane")))

    def test_parallel_refuses_a_bad_branch_name_in_the_viewer(self):
        self.git_repository()
        self.run_manager("parallel", str(self.project), "bad..name", CLAUDE_SESSIONS_CLONES=str(self.root / "clones"))
        self.assertFalse((self.root / "clones").exists())
        self.assertTrue(any("not a branch name" in call for call in self.tmux_calls("respawn-pane")))

    def test_review_refuses_what_is_not_a_pull_request_number(self):
        self.run_manager("review", str(self.project), "12; rm -rf /")
        self.assertTrue(any("not a pull request number" in call for call in self.tmux_calls("respawn-pane")))

    def log_lines(self) -> list[str]:
        path = self.state / "claude-sessions/log"
        return path.read_text().splitlines() if path.exists() else []

    def test_every_command_leaves_a_log_line_with_its_outcome(self):
        self.run_manager("view", SESSION_A)
        self.run_manager("label-state", "purple")
        lines = self.log_lines()
        self.assertRegex(lines[0], r"^\S+ INFO manager: view " + SESSION_A + r" -> exit 0 \(\d+ ms\)$")
        self.assertRegex(lines[-1], r" ERROR manager: label-state purple -> exit 1 ")

    def test_claude_arguments_to_new_are_counted_not_logged(self):
        self.run_manager("new", str(self.root), "fix the secret thing")
        line = next(line for line in self.log_lines() if " manager: new " in line)
        self.assertNotIn("secret", line)
        self.assertIn("(+1 claude arguments)", line)

    def test_errors_of_a_background_run_reach_the_log(self):
        self.run_manager("view", "no-such-session", CLAUDE_SESSIONS_LOG_STDERR="1")
        deadline = time.time() + 3
        while time.time() < deadline and not any("has no tmux pane" in line for line in self.log_lines()):
            time.sleep(0.05)
        self.assertTrue(any(" ERROR manager: claude-sessions: session no-such-session has no tmux pane" in line for line in self.log_lines()))

    def test_the_log_rotates_past_its_size(self):
        log = self.state / "claude-sessions/log"
        log.parent.mkdir(parents=True, exist_ok=True)
        log.write_text("x" * 1_100_000)
        self.run_manager("log", "test", "INFO", "after rotation")
        self.assertTrue((self.state / "claude-sessions/log.1").exists())
        self.assertEqual(len(self.log_lines()), 1)

    def test_a_failed_launch_explains_itself_in_the_pane(self):
        self.run_manager("new", str(self.root), "--effort", "max")
        launch = next(call for call in self.calls("tmux") if "claude $argv" in call)
        self.assertIn("claude exited with status $code", launch)
        self.assertIn("claude-sessions doctor", launch)

    def test_restart_refuses_an_unknown_effort_before_touching_the_session(self):
        result = self.run_manager("restart", SESSION_A, "turbo")
        self.assertEqual(result.returncode, 1)
        self.assertIn("unknown effort turbo", result.stderr)
        self.assertEqual(self.tmux_calls("send-keys"), [])

    def test_show_from_the_claude_workspace_goes_back(self):
        self.run_manager("show", SHIM_WS="6")
        self.assertIn('hyprctl dispatch hl.dsp.focus({ workspace = "previous" })', self.calls("hyprctl"))
        self.assertEqual(self.calls("setsid"), [])

    def test_show_elsewhere_hides_the_bar_and_opens_the_window(self):
        self.shim("pkill", "")
        self.run_manager("show", SHIM_WS="2")
        self.assertIn("pkill -x -USR1 waybar", self.calls("pkill"))
        self.assertEqual(self.calls("setsid"), [f"setsid -f kitty --class claude-sessions --config {self.home}/.config/kitty/kitty.conf "
                                                f"--config {self.home}/.config/kitty/claude-sessions.conf -e claude-sessions"])

    def test_the_first_show_of_the_day_opens_the_digest_once(self):
        self.shim("pkill", "")
        self.run_manager("show", SHIM_WS="2")
        self.run_manager("show", SHIM_WS="2")
        digests = [call for call in self.tmux_calls("respawn-pane") if "claude-sessions-sidebar digest" in call]
        self.assertEqual(len(digests), 1)

    def test_reset_clears_the_daemon_and_the_caches(self):
        cache = self.home / ".cache/claude"
        cache.mkdir(parents=True)
        (cache / "statusline-x.json").write_text("{}")
        self.run_manager("reset")
        self.assertIn("osd-show sessions-clear", self.calls("osd-show"))
        self.assertFalse((cache / "statusline-x.json").exists())

    def test_unknown_subcommand_prints_usage(self):
        result = self.run_manager("bogus")
        self.assertEqual(result.returncode, 2)
        self.assertIn("usage:", result.stderr)


if __name__ == "__main__":
    unittest.main()


class HandoffTests(ManagerTests):
    """Remote Control hand-off: /rc typed at an idle prompt, queued while busy, only where needed."""

    def mark_remote(self, session: str, remote: bool):
        cache = self.home / ".cache/claude"
        cache.mkdir(parents=True, exist_ok=True)
        (cache / f"statusline-{session}.json").write_text(json.dumps({"remote": remote}))

    def test_handoff_types_rc_at_an_idle_prompt(self):
        self.run_manager("handoff", SHIM_PANE_TEXT="done\n❯ \n")
        self.assertEqual(self.tmux_calls("send-keys"), ["tmux send-keys -t %4 /rc", "tmux send-keys -t %4 Enter"])

    def test_handoff_queues_rc_while_claude_works(self):
        self.run_manager("handoff", SHIM_PANE_TEXT="✻ Flowing… (11s)\n❯ \n")
        self.assertEqual(self.tmux_calls("send-keys"), ["tmux send-keys -t %4 /rc", "tmux send-keys -t %4 C-x Enter"])

    def test_unlock_takes_back_only_what_lock_handed_off(self):
        # Session A starts on Remote Control by hand; B is local.
        self.snapshot.write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "state": "waiting", "project": "a", "cwd": str(self.project), "tmux_pane": "%4"},
            {"session": SESSION_B, "state": "waiting", "project": "b", "cwd": str(self.project), "tmux_pane": "%5"}]}))
        cache = self.home / ".cache/claude"
        cache.mkdir(parents=True, exist_ok=True)
        (cache / f"statusline-{SESSION_A}.json").write_text('{"remote": true}')
        self.run_manager("handoff", "lock", SHIM_PANE_TEXT="done\n❯ \n")
        self.assertEqual((self.runtime / "claude-sessions.handed-off").read_text().split(), [SESSION_B])
        locked = len(self.tmux_calls("send-keys"))
        self.run_manager("handoff", "unlock", SHIM_PANE_TEXT="done\n❯ \n")
        unlocked = self.tmux_calls("send-keys")[locked:]
        self.assertTrue(unlocked and all("-t %5" in call for call in unlocked))
        self.assertFalse((self.runtime / "claude-sessions.handed-off").exists())

    def test_handoff_refuses_an_unknown_mode(self):
        self.assertEqual(self.run_manager("handoff", "sideways").returncode, 1)

    def test_handoff_skips_sessions_already_remote_and_off_takes_them_back(self):
        self.mark_remote(SESSION_A, True)
        result = self.run_manager("handoff", SHIM_PANE_TEXT="done\n❯ \n")
        self.assertEqual(self.tmux_calls("send-keys"), [])
        self.assertIn("for 0 session", result.stdout)
        self.run_manager("handoff", "off", SHIM_PANE_TEXT="done\n❯ \n")
        self.assertEqual(self.tmux_calls("send-keys")[:1], ["tmux send-keys -t %4 /rc"])

    def test_remote_toggles_one_session(self):
        self.run_manager("remote", SESSION_A, SHIM_PANE_TEXT="done\n❯ \n")
        self.assertEqual(self.tmux_calls("send-keys")[0], "tmux send-keys -t %4 /rc")

    def test_manager_gets_a_tagged_viewer_pane(self):
        (self.root / "roles").write_text("%10 sidebar\n")
        self.run_manager("prepare")
        splits = self.tmux_calls("split-window")
        self.assertEqual(len(splits), 1)
        self.assertIn("claude-sessions idle", splits[0])
        self.assertEqual([line.split()[1] for line in (self.root / "roles").read_text().splitlines()[1:]], ["viewer"])

    def test_a_manager_without_role_tags_is_refused_with_the_fix(self):
        (self.root / "roles").write_text("")
        result = self.run_manager("prepare")
        self.assertEqual(result.returncode, 1)
        self.assertIn("tmux kill-session -t claude-manager", result.stderr)
