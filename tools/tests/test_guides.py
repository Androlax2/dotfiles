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
