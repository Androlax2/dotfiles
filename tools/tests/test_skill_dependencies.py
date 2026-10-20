import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest


REPOSITORY = Path(__file__).resolve().parents[2]


class SkillDependencyTests(unittest.TestCase):
    def test_package_setup_installs_shared_helpers_for_inline_execution(self):
        for installed_skills in ("", "obra/superpowers executing-plans\n"):
            with self.subTest(installed_skills=installed_skills):
                with tempfile.TemporaryDirectory() as temporary:
                    folder = Path(temporary)
                    inventory = folder / "installed-skills"
                    inventory.write_text(installed_skills)
                    log = folder / "install-commands"
                    npx = folder / "npx"
                    npx.write_text(
                        '#!/bin/sh\n'
                        'printf "%s " "$@" >> "$SKILLS_INSTALL_LOG"\n'
                        'printf "\\n" >> "$SKILLS_INSTALL_LOG"\n'
                    )
                    npx.chmod(0o700)
                    environment = dict(os.environ, PATH=str(folder) + ":" + os.environ["PATH"],
                                       SKILLS_INSTALL_LOG=str(log))
                    overrides = [
                        f"installed_{manifest.stem}=cat {shlex.quote(str(manifest))}"
                        for manifest in (REPOSITORY / "packages").glob("*.txt")
                        if manifest.stem != "skills"
                    ]
                    result = subprocess.run(
                        ["make", "--no-print-directory", "global-packages",
                         *overrides, f"installed_skills=cat {shlex.quote(str(inventory))}"],
                        cwd=REPOSITORY, env=environment, capture_output=True, text=True, timeout=30,
                    )
                    self.assertEqual(result.returncode, 0, result.stderr)
                    commands = [shlex.split(line) for line in log.read_text().splitlines()]
                    superpowers = [command for command in commands if "obra/superpowers" in command]
                    self.assertEqual(len(superpowers), 1, commands)
                    command = superpowers[0]
                    requested_agents = [command[index + 1] for index, argument in enumerate(command)
                                        if argument == "--agent"]
                    self.assertIn("codex", requested_agents,
                                  "shared helpers must also be installed in ~/.agents/skills")
                    requested_skills = [command[index + 1] for index, argument in enumerate(command)
                                        if argument == "--skill"]
                    self.assertIn("subagent-driven-development", requested_skills,
                                  "executing-plans needs the sibling workspace, brief and review scripts")


if __name__ == "__main__":
    unittest.main()
