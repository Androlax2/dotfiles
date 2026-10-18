"""Guides: ask-to-guide (inside the sandbox), claude-guide (outside, in the docked panel), the daemon's queue."""

import builtins
import importlib.machinery
import importlib.util
import json
import os
import pathlib
import subprocess
import time

from support import BIN, OSD, Sandbox

STEPS = """# Search Console key
## Turn on the API
1. Open https://console.cloud.google.com/ and pick the project
2. Enable the API
   open: https://console.cloud.google.com/apis/library
## Service account
- Copy the new account's email
  copy: review@example.iam.gserviceaccount.com
  It ends in iam.gserviceaccount.com.
Paste it here
  ask: Which email did it get?
Run the check
  run: make search-console
"""


def load(path, name):
    loader = importlib.machinery.SourceFileLoader(name, str(path))
    module = importlib.util.module_from_spec(importlib.util.spec_from_loader(name, loader))
    loader.exec_module(module)
    return module


class AskToGuideTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.guides = self.root / "guides"
        self.fifo = self.runtime / "osd.fifo"

    def listen(self):
        os.mkfifo(self.fifo)
        reader = subprocess.Popen(["cat", str(self.fifo)], stdout=subprocess.PIPE, text=True)
        holder = subprocess.Popen(["sleep", "30"], stdout=open(self.fifo, "w"))
        self.addCleanup(reader.kill)
        self.addCleanup(holder.kill)
        return reader, holder

    def ask(self, steps, *arguments, wait="5"):
        asking = subprocess.Popen([str(BIN / "ask-to-guide"), *arguments], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, text=True, cwd=self.root,
                                  env={**self.env, "CLAUDE_GUIDES_DIR": str(self.guides), "ASK_TO_GUIDE_WAIT_S": wait})
        asking.stdin.write(steps)
        asking.stdin.close()
        return asking

    def filed(self) -> dict:
        deadline = time.time() + 3
        while time.time() < deadline:
            found = list(self.guides.glob("*.guide")) if self.guides.exists() else []
            if found:
                return json.loads(found[0].read_text())
            time.sleep(0.05)
        self.fail("no guide filed")

    def answer(self, guide: dict, result: dict):
        (self.guides / f"{guide['id']}.result").write_text(json.dumps(result))

    def test_steps_are_read_with_sections_links_values_questions_and_commands(self):
        title, steps = load(BIN / "ask-to-guide", "ask_to_guide").parse_steps(STEPS)
        self.assertEqual(title, "Search Console key")
        self.assertEqual([step["text"].split("\n")[0] for step in steps],
                         ["Open https://console.cloud.google.com/ and pick the project", "Enable the API",
                          "Copy the new account's email", "Paste it here", "Run the check"])
        self.assertEqual([step["section"] for step in steps], ["Turn on the API"] * 2 + ["Service account"] * 3)
        self.assertEqual(steps[1]["open"], "https://console.cloud.google.com/apis/library")
        self.assertEqual(steps[2]["copy"], "review@example.iam.gserviceaccount.com")
        self.assertEqual(steps[2]["text"], "Copy the new account's email\nIt ends in iam.gserviceaccount.com.")
        self.assertEqual((steps[3]["ask"], steps[4]["run"]), ("Which email did it get?", "make search-console"))

    def test_a_guide_is_filed_announced_and_its_answers_come_back(self):
        reader, holder = self.listen()
        asking = self.ask(STEPS)
        guide = self.filed()
        self.assertEqual((guide["title"], len(guide["steps"]), guide["cwd"]), ("Search Console key", 5, str(self.root)))
        self.answer(guide, {"status": "done", "total": 5, "answers": [{"step": 4, "question": "Which email?", "answer": "a@b"}],
                            "runs": [{"step": 5, "command": "make x", "status": "ran", "exit": 0, "output": "\x1b[32mok\x1b[0m"}]})
        stdout, _ = asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 0)
        self.assertIn("Theo answered: a@b", stdout)
        self.assertIn("ok\n[exit 0]", stdout)
        holder.kill()
        self.assertEqual(reader.communicate(timeout=5)[0].strip(), f"guide-request {guide['id']}")

    def test_stuck_exits_3_with_the_note(self):
        self.listen()
        asking = self.ask(STEPS)
        guide = self.filed()
        self.answer(guide, {"status": "stuck", "step": 2, "total": 5, "step_text": "Enable the API", "note": "403 on enable"})
        stdout, _ = asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 3)
        self.assertIn("stuck at step 2 of 5: Enable the API", stdout)
        self.assertIn("His note: 403 on enable", stdout)

    def test_the_wait_running_out_says_where_theo_is(self):
        self.listen()
        asking = self.ask(STEPS, wait="1")
        guide = self.filed()
        (self.guides / f"{guide['id']}.progress").write_text(json.dumps({"step": 3, "total": 5, "done": 2}))
        stdout, _ = asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 124)
        self.assertIn("on step 3 of 5 (2 done)", stdout)
        self.assertIn(f"ask-to-guide --wait {guide['id']}", stdout)

    def test_without_a_daemon_it_exits_126_and_leaves_no_guide(self):
        asking = self.ask(STEPS)
        asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 126)
        self.assertEqual(list(self.guides.glob("*.guide")), [])

    def test_no_steps_exits_2(self):
        asking = self.ask("# only a title\n")
        asking.communicate(timeout=5)
        self.assertEqual(asking.returncode, 2)


class ClaudeGuideTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.guide_module = load(BIN / "claude-guide", "claude_guide")
        self.guide_module.GUIDES_DIR = str(self.root)
        self.guide_module.approval.send = lambda line: None
        self.guide_module.approval.log = lambda level, message: None

    def file(self, steps=None, guide_id="aaaaaaaaaaaa", **fields) -> str:
        guide = {"id": guide_id, "title": "Key", "cwd": str(self.root), "requested_at": int(time.time()),
                 "steps": steps or [{"text": "Open https://example.com/a and log in"}, {"text": "Answer", "ask": "Which email?"},
                                    {"text": "Check", "run": "make check"}], **fields}
        (self.root / f"{guide_id}.guide").write_text(json.dumps(guide))
        return guide_id

    def open_guide(self, **fields):
        guide_id = self.file(**fields)
        return self.guide_module.Guide(guide_id, self.guide_module.read_guide(guide_id))

    def test_unusable_guides_are_refused(self):
        for guide_id, fields in (("../etc", {}), ("bbbbbbbbbbbb", {"requested_at": 1}), ("cccccccccccc", {"steps": [{"text": " "}]}),
                                 ("dddddddddddd", {"steps": [{"text": "x", "run": ["rm", "-rf"]}]})):
            if guide_id != "../etc":
                self.file(guide_id=guide_id, **fields)
            with self.subTest(guide_id), self.assertRaises((ValueError, OSError)):
                self.guide_module.read_guide(guide_id)

    def test_done_moves_to_the_next_step_not_done_and_reports_the_end(self):
        guide = self.open_guide()
        guide.move(1)
        self.assertFalse(guide.mark_done())
        self.assertEqual(guide.position, 2)
        self.assertFalse(guide.mark_done())
        self.assertEqual(guide.position, 0)  # back to the one skipped
        self.assertTrue(guide.mark_done())

    def test_the_box_is_the_answer_on_a_question_and_a_note_elsewhere(self):
        guide = self.open_guide()
        guide.submit("  login   done  ")
        guide.move(1)
        guide.submit("a@b")
        self.assertEqual((guide.notes, guide.answers), ({0: "login done"}, {1: "a@b"}))
        guide.move(-1)
        self.assertEqual(guide.typed(), "login done")
        guide.submit("")
        self.assertEqual(guide.notes, {})

    def test_typing_pasting_and_enter_go_through_the_real_screen(self):
        import pty, select
        guide_id = self.file(steps=[{"text": "Log in"}, {"text": "Answer", "ask": "Which email?"}])
        pid, fd = pty.fork()
        if pid == 0:
            os.environ.update(CLAUDE_GUIDES_DIR=str(self.root), TERM="xterm-256color")
            os.execv(str(BIN / "claude-guide"), ["claude-guide", guide_id])

        def pump(seconds):
            end = time.time() + seconds
            while time.time() < end:
                if select.select([fd], [], [], 0.05)[0]:
                    try:
                        os.read(fd, 65536)
                    except OSError:
                        return
        for keys in (b"logged in", b"\r", b"\x1b[200~a@b\nc\x1b[201~", b"\r"):
            pump(0.6)
            os.write(fd, keys)
        pump(2.5)
        os.waitpid(pid, 0)
        result = json.loads((self.root / f"{guide_id}.result").read_text())
        self.assertEqual(result["status"], "done")
        self.assertEqual(result["notes"], [{"step": 1, "note": "logged in"}])
        self.assertEqual(result["answers"], [{"step": 2, "question": "Which email?", "answer": "a@b c"}])

    def test_hidden_characters_are_shown_escaped_and_flagged(self):
        guide = self.open_guide(steps=[{"text": "Run‮ this", "copy": "a\x1b[2Kb"}])
        lines = self.guide_module.render_step(guide, 50)
        text = "\n".join(line for line, _ in lines)
        self.assertNotIn("‮", text)
        self.assertNotIn("\x1b", text)
        self.assertIn("\\u202e", text)
        self.assertIn("hidden characters", text)

    def test_only_http_links_open(self):
        link_of = self.guide_module.link_of
        self.assertEqual(link_of({"text": "Go to https://example.com/x."}), "https://example.com/x.")
        self.assertEqual(link_of({"text": "x", "open": "javascript:alert(1)"}), "")
        self.assertEqual(link_of({"text": "x", "open": "file:///etc/passwd"}), "")

    def test_a_run_step_is_declined_by_default(self):
        guide = self.open_guide()
        guide.position = 2
        ran = []
        self.guide_module.approval.run = lambda *arguments: ran.append(arguments) or 0
        original_input = builtins.input
        self.addCleanup(setattr, builtins, "input", original_input)
        builtins.input = lambda prompt="": ""
        self.guide_module.run_step(guide)
        self.assertEqual(ran, [])
        self.assertEqual(guide.runs[2]["status"], "declined")

    def test_an_approved_run_step_records_its_exit_and_output(self):
        guide = self.open_guide()
        guide.position = 2
        self.guide_module.approval.run = lambda command, cwd, log_path: pathlib.Path(log_path).write_text("checked\n") and 4
        self.guide_module.approval.LOG_DIR = str(self.root)
        answers = iter(["y", ""])
        original_input = builtins.input
        self.addCleanup(setattr, builtins, "input", original_input)
        builtins.input = lambda prompt="": next(answers)
        self.guide_module.run_step(guide)
        self.assertEqual({key: guide.runs[2][key] for key in ("status", "exit", "output")}, {"status": "ran", "exit": 4, "output": "checked"})

    def test_the_result_carries_answers_runs_and_where_theo_stopped(self):
        guide = self.open_guide()
        guide.answers[1] = "a@b"
        guide.runs[2] = {"status": "declined", "command": "make check"}
        guide.position = 1
        self.guide_module.save_progress(guide)
        self.guide_module.finish(guide, "stuck", "no email shown")
        result = json.loads((self.root / "aaaaaaaaaaaa.result").read_text())
        self.assertEqual((result["status"], result["step"], result["note"]), ("stuck", 2, "no email shown"))
        self.assertEqual(result["answers"], [{"step": 2, "question": "Which email?", "answer": "a@b"}])
        self.assertEqual(result["runs"][0]["step"], 3)
        self.assertFalse((self.root / "aaaaaaaaaaaa.progress").exists())


class GuideQueueTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.registry = load(OSD / "claude_registry.py", "claude_registry_guides")
        self.registry.LOG_PATH = os.devnull
        self.registry.GUIDES_DIR = str(self.root)
        self.windows, self.notices = [], []
        self.is_gaming = False

        class Window:
            def __init__(self, command):
                self.command, self.closed = command, False

            def poll(self):
                return 0 if self.closed else None

        class Banners:
            def raise_notice(banners, notice):
                self.notices.append(notice["id"])

            def drop(banners, key):
                self.notices.append("dropped " + key)

        self.registry.gaming = lambda: self.is_gaming
        original = self.registry.subprocess.Popen
        self.addCleanup(setattr, self.registry.subprocess, "Popen", original)
        self.registry.subprocess.Popen = lambda command, **options: self.windows.append(Window(command)) or self.windows[-1]
        self.guides = self.registry.GuideRequests(Banners(), lambda seconds, callback: None)

    def test_a_guide_opens_in_a_floating_window_of_its_own_one_at_a_time(self):
        with self.assertRaises(ValueError):
            self.guides.request("../../etc")
        self.guides.request("aaaaaaaaaaaa")
        self.guides.request("bbbbbbbbbbbb")
        self.assertEqual(len(self.windows), 1)
        self.assertEqual(self.windows[0].command[:3], ["kitty", "--class", "claude-guide"])
        self.assertEqual(self.windows[0].command[-1], "aaaaaaaaaaaa")

    def test_while_gaming_the_guide_waits_then_opens(self):
        self.is_gaming = True
        self.guides.request("aaaaaaaaaaaa")
        self.guides.tick()
        self.assertEqual((self.windows, self.notices), ([], []))
        self.is_gaming = False
        self.guides.tick()
        self.assertEqual(len(self.windows), 1)

    def test_closing_the_window_tells_claude_where_theo_stopped_and_opens_the_next(self):
        self.guides.request("aaaaaaaaaaaa")
        self.guides.request("bbbbbbbbbbbb")
        (self.root / "aaaaaaaaaaaa.progress").write_text(json.dumps({"step": 2, "total": 5, "done": 1}))
        self.windows[0].closed = True
        self.guides.tick()
        result = json.loads((self.root / "aaaaaaaaaaaa.result").read_text())
        self.assertEqual((result["status"], result["step"], result["total"]), ("closed", 2, 5))
        self.assertIn("dropped notice:guide-aaaaaaaaaaaa", self.notices)
        self.assertEqual((self.guides.current, self.windows[1].command[-1]), ("bbbbbbbbbbbb", "bbbbbbbbbbbb"))
