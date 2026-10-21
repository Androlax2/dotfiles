import os
from pathlib import Path
import subprocess
import tempfile
import unittest


REPOSITORY = Path(__file__).resolve().parents[2]


class DiskReportTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name)
        self.clones = self.home / "IdeaProjects/.parallel/dotfiles"

    def run_make_disk(self):
        environment = dict(os.environ, HOME=str(self.home))
        environment.pop("COCKPIT_CLONES", None)
        return subprocess.run(["make", "--no-print-directory", "-C", str(REPOSITORY), "disk"],
                              env=environment, capture_output=True, text=True, timeout=120)

    def add_clone(self, name):
        build = self.clones / name / "apps/target"
        build.mkdir(parents=True)
        (build / "artifact").write_bytes(b"x" * 8192)
        return build

    def test_counts_agent_clones_and_lists_their_build_output(self):
        build = self.add_clone("parallel-1")
        self.add_clone("todo-2")
        result = self.run_make_disk()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("2 clones", result.stdout)
        self.assertIn(str(build), result.stdout)

    def test_report_leaves_build_output_in_place(self):
        build = self.add_clone("parallel-1")
        self.run_make_disk()
        self.assertEqual((build / "artifact").read_bytes(), b"x" * 8192)

    def test_missing_folders_do_not_fail_the_report(self):
        result = self.run_make_disk()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("agent clones", result.stdout)


if __name__ == "__main__":
    unittest.main()
