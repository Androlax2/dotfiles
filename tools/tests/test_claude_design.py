"""claude-design, the command a design session runs to link its card to a Claude Design canvas."""

import json
import subprocess

from support import BIN, SESSION_A, Sandbox


class ClaudeDesignTests(Sandbox):
    def design(self, *arguments, session=SESSION_A):
        env = {**self.env, "CLAUDE_DESIGNS_DIR": str(self.root / "designs"), "CLAUDE_CODE_SESSION_ID": session}
        return subprocess.run([str(BIN / "claude-design"), *arguments], env=env, capture_output=True, text=True)

    def test_a_canvas_is_linked_to_the_session_with_its_title(self):
        self.assertEqual(self.design("link", "https://claude.ai/artifact/d31175a3-dc24-4166-aa81-2ccdf5efe83a", "Checkout  redesign").returncode, 0)
        link = json.loads((self.root / "designs" / f"{SESSION_A}.json").read_text())
        self.assertEqual((link["url"], link["title"]), ("https://claude.ai/artifact/d31175a3-dc24-4166-aa81-2ccdf5efe83a", "Checkout redesign"))

    def test_a_chat_address_becomes_the_canvas_address(self):
        self.design("link", "https://claude.ai/chat/7efffff6-0466-41ab-83d1-3f1d403d357d?artifact=d31175a3-dc24-4166-aa81-2ccdf5efe83a", "Canvas")
        self.assertEqual(self.design("show").stdout.strip(), "https://claude.ai/artifact/d31175a3-dc24-4166-aa81-2ccdf5efe83a")

    def test_other_addresses_and_missing_titles_are_refused(self):
        for arguments in (("link", "https://evil.example/artifact/abcdefgh", "x"), ("link", "https://claude.ai/artifact/abcdefgh12", " ")):
            with self.subTest(arguments):
                self.assertEqual(self.design(*arguments).returncode, 2)
        self.assertFalse((self.root / "designs").exists())

    def test_unlink_removes_the_canvas_from_the_card(self):
        self.design("link", "https://claude.ai/artifact/abcdefgh12", "Canvas")
        self.design("unlink")
        self.assertEqual(self.design("show").stdout.strip(), "No canvas linked to this session.")
