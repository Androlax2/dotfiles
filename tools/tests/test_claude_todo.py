"""claude-todo, the command sessions use inside the sandbox to update Theo's todo list."""

import json
import subprocess

from support import BIN, SESSION_A, Sandbox


class ClaudeTodoTests(Sandbox):
    """claude-todo, the command sessions use inside the sandbox: reads the list, appends updates."""

    def claude_todo(self, *arguments, cwd=None):
        env = {**self.env, "CLAUDE_TODO_PATH": str(self.state / "todo.json"), "CLAUDE_TODO_UPDATES": str(self.root / "updates.jsonl"),
               "CLAUDE_CODE_SESSION_ID": SESSION_A}
        return subprocess.run([str(BIN / "claude-todo"), *arguments], cwd=cwd or self.root, env=env, capture_output=True, text=True)

    def updates(self):
        return [json.loads(line) for line in (self.root / "updates.jsonl").read_text().splitlines()]

    def test_start_and_done_append_an_update_with_the_session(self):
        self.assertEqual(self.claude_todo("start", "0123abcd").returncode, 0)
        self.assertEqual(self.claude_todo("done", "o/runway#12").returncode, 0)
        self.assertEqual([(update["action"], update["id"], update["session"]) for update in self.updates()],
                         [("doing", "0123abcd", SESSION_A), ("done", "o/runway#12", SESSION_A)])

    def test_a_malformed_id_is_refused(self):
        result = self.claude_todo("done", "rm -rf")
        self.assertEqual(result.returncode, 2)
        self.assertFalse((self.root / "updates.jsonl").exists())

    def test_list_shows_the_items_of_this_directorys_repository(self):
        checkout = self.root / "runway"
        (checkout / ".git").mkdir(parents=True)
        (self.state / "todo.json").write_text(json.dumps({"items": [
            {"id": "0123abcd", "project": "runway", "text": "deploy", "status": "doing"},
            {"id": "4567cdef", "project": "homelab", "text": "postgres", "status": "todo"}], "issues": {}}))
        listed = self.claude_todo("list", cwd=checkout).stdout
        self.assertIn("0123abcd", listed)
        self.assertNotIn("4567cdef", listed)
        self.assertIn("4567cdef", self.claude_todo("list", "--all", cwd=checkout).stdout)
