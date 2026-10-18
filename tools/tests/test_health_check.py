"""The weekly health check (claude-health-check): a report always, a banner only on problems."""

import json
import os
import subprocess
import time

from support import BIN, Sandbox


class HealthCheckTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.shim("make", 'echo "make $*"; [[ $* == *" test" ]] && exit "${SHIM_TESTS_EXIT:-0}"; exit 0')
        self.shim("claude-sessions", '[[ $1 == doctor ]] && { echo "1 problem(s)"; exit "${SHIM_DOCTOR_EXIT:-0}"; }; exit 0')
        recent = time.strftime("%a %Y-%m-%d %H:%M:%S %Z", time.localtime(time.time() - 3600))
        self.shim("systemctl", f'[[ $* == *Result* ]] && echo "${{SHIM_BACKUP:-success}}"; [[ $* == *ExecMainExitTimestamp* ]] && echo "${{SHIM_BACKUP_AT:-{recent}}}"; exit 0')
        self.shim("df", 'echo "Use% Mounted on"; echo " ${SHIM_DISK:-40%} /"; echo " 40% /home"')

    def run_check(self, **env) -> subprocess.CompletedProcess:
        return subprocess.run([str(BIN / "claude-health-check")], capture_output=True, text=True,
                              env={**self.env, "CLAUDE_HEALTH_DOTFILES": str(self.root), **env})

    def report(self) -> str:
        return next((self.state / "claude-sessions").glob("health-*.txt")).read_text()

    def logged(self) -> list[str]:
        return [call for call in self.calls("claude-sessions") if call.startswith("claude-sessions log")]

    def test_a_quiet_week_writes_a_report_and_logs_all_good(self):
        self.assertEqual(self.run_check().returncode, 0)
        self.assertIn("== make check (drift, for reading)", self.report())
        self.assertTrue(self.logged()[-1].startswith("claude-sessions log health INFO weekly check: all good"))

    def test_problems_are_named_logged_and_shown_as_a_banner(self):
        fifo = self.runtime / "osd.fifo"
        os.mkfifo(fifo)
        reader = subprocess.Popen(["cat", str(fifo)], stdout=subprocess.PIPE, text=True)
        holder = subprocess.Popen(["sleep", "30"], stdout=open(fifo, "w"))
        self.addCleanup(reader.kill)
        self.addCleanup(holder.kill)
        result = self.run_check(SHIM_DOCTOR_EXIT="1", SHIM_TESTS_EXIT="2", SHIM_BACKUP="exit-code", SHIM_DISK="95%")
        self.assertEqual(result.returncode, 1)
        line = self.logged()[-1]
        for problem in ("doctor: 1 problem(s)", "tests failing", "backup: exit-code", "/ at 95%"):
            self.assertIn(problem, line)
        time.sleep(0.3)
        holder.kill()
        notice = reader.communicate(timeout=5)[0].splitlines()[0]
        self.assertTrue(notice.startswith("notice "))
        self.assertEqual(json.loads(notice[len("notice "):])["title"], "Weekly check: 4 problem(s)")

    def test_an_old_backup_is_a_problem(self):
        self.run_check(SHIM_BACKUP_AT="Mon 2026-09-01 10:00:00 CEST")
        self.assertIn("backup older than two days", self.logged()[-1])
