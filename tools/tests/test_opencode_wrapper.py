"""Run with: python3 -m unittest discover -s tools/tests -v."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


WRAPPER = Path(__file__).resolve().parents[2] / "config/bin/.local/bin/opencode"


class OpenCodeWrapperTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="opencode-wrapper-")
        self.addCleanup(self.directory.cleanup)
        self.home = Path(self.directory.name)
        self.bin = self.home / ".local/bin"
        self.bin.mkdir(parents=True)
        # Replace only the sandbox launcher and application, not the wrapper.
        # A real nested Greywall launch requires host namespace privileges.
        greywall = self.bin / "greywall"
        greywall.write_text('''#!/usr/bin/env python3
import json, os, sys
args = sys.argv[1:]
boundary = args.index("--")
os.environ["TEST_GREYWALL_FLAGS"] = json.dumps(args[:boundary])
os.execvp(args[boundary + 1], args[boundary + 1:])
''')
        greywall.chmod(0o755)
        application = self.home / ".opencode/bin/opencode"
        application.parent.mkdir(parents=True)
        application.write_text('''#!/usr/bin/env python3
import json, os, sys
print(json.dumps({"args": sys.argv[1:], "state": os.environ.get("XDG_STATE_HOME"),
    "home": os.environ["HOME"], "data": os.environ.get("XDG_DATA_HOME"),
    "config": os.environ.get("XDG_CONFIG_HOME"), "cache": os.environ.get("XDG_CACHE_HOME"),
    "flags": json.loads(os.environ["TEST_GREYWALL_FLAGS"])}))
''')
        application.chmod(0o755)
        # Also exercise the legacy wrapper safely: it hard-coded /home/theo.
        # Rebase that home prefix into the fixture, leaving its logic unchanged.
        self.wrapper = self.home / "wrapper"
        self.wrapper.write_text(WRAPPER.read_text().replace("/home/theo/", f"{self.home}/"))

    def run_wrapper(self, *args):
        env = dict(os.environ, HOME=str(self.home),
                   XDG_STATE_HOME=str(self.home / "host-state"),
                   XDG_DATA_HOME=str(self.home / "data"),
                   XDG_CONFIG_HOME=str(self.home / "config"),
                   XDG_CACHE_HOME=str(self.home / "cache"))
        result = subprocess.run(["bash", str(self.wrapper), *args], env=env,
                                text=True, capture_output=True, check=True)
        return json.loads(result.stdout)

    def test_registration_uses_sandbox_tmp_not_shared_host_state(self):
        result = self.run_wrapper("debug", "paths", "state")
        self.assertEqual(result["state"], "/tmp/opencode-greywall-state")
        self.assertFalse((self.home / "host-state").exists())

    def test_preserves_application_paths_and_arguments(self):
        args = ["run", "a prompt with spaces", "", "--standalone"]
        result = self.run_wrapper(*args)
        self.assertEqual(result["args"], args)
        self.assertEqual(result["home"], str(self.home))
        for key in ("data", "config", "cache"):
            self.assertEqual(result[key], str(self.home / key))

    def test_keeps_opencode_sandbox_profile_and_proxy_identity(self):
        flags = self.run_wrapper("--version")["flags"]
        self.assertIn("--profile", flags)
        self.assertEqual(flags[flags.index("--profile") + 1], "opencode")
        self.assertIn("--proxy-user", flags)
        self.assertEqual(flags[flags.index("--proxy-user") + 1], "opencode")
        self.assertEqual(flags[flags.index("--allow-path") + 1],
                         str(self.home / ".cache/chrome-sandbox"))


if __name__ == "__main__":
    unittest.main()
