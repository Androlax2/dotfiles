import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest


REPOSITORY = Path(__file__).resolve().parents[2]


class TestEntrypointTests(unittest.TestCase):
    def run_make_test(self, fail_build=False):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            cargo = folder / "cargo"
            cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$CARGO_TEST_LOG"\n'
                             'if [ "$1" = build ] && [ "$CARGO_TEST_FAIL_BUILD" = 1 ]; then exit 9; fi\n')
            cargo.chmod(0o700)
            log = folder / "commands"
            environment = dict(os.environ, PATH=str(folder) + ":" + os.environ["PATH"],
                               CARGO_TEST_LOG=str(log), CARGO_TEST_FAIL_BUILD="1" if fail_build else "0")
            result = subprocess.run(["make", "--no-print-directory", "-f", str(REPOSITORY / "Makefile"), "test"],
                                    cwd=REPOSITORY, env=environment, capture_output=True, text=True, timeout=30)
            calls = [shlex.split(line) for line in log.read_text().splitlines()] if log.exists() else []
            return result, calls

    def test_test_target_builds_the_real_classifier_before_running_tests(self):
        result, calls = self.run_make_test()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 2, calls)
        self.assertEqual(calls[0][0], "build")
        self.assertIn("--release", calls[0])
        self.assertEqual(calls[0][calls[0].index("--bin") + 1], "osd")
        self.assertEqual(calls[1][0], "test")
        self.assertIn("--release", calls[1])

    def test_build_failure_stops_the_test_target(self):
        result, calls = self.run_make_test(fail_build=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([arguments[0] for arguments in calls], ["build"])


if __name__ == "__main__":
    unittest.main()
