#!/usr/bin/env python3
"""Build a made-up Claude Code cockpit for the README screenshots (tools/capture-claude.sh).

    build.py <root>

Everything the cockpit's column reads, invented, under <root>: home/ (statusline facts, history,
plan name, parallel clones), run/ (the session snapshot, plan usage, the viewed session),
state/ (todo list), repos/ (git checkouts with their branch and a github.com origin), bin/ (a gh
that answers pull requests and issues from here, a no-op claude-sessions), and status.json for
status.claude.com. Pointing HOME, XDG_RUNTIME_DIR, XDG_STATE_HOME, PATH and CLAUDE_STATUS_URL at
it shows none of the real sessions, tasks or costs.
"""

import json
import os
import stat
import subprocess
import sys
import time
from datetime import datetime, timedelta, timezone

NOW = time.time()


def iso(seconds_from_now: float) -> str:
    return (datetime.now(timezone.utc) + timedelta(seconds=seconds_from_now)).strftime("%Y-%m-%dT%H:%M:%SZ")


def write(path: str, content: str, executable: bool = False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as file:
        file.write(content)
    if executable:
        os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)


def checkout(path: str, branch: str, repository: str):
    os.makedirs(path, exist_ok=True)
    subprocess.run(["git", "init", "-q", "-b", branch, path], check=True)
    subprocess.run(["git", "-C", path, "remote", "add", "origin", f"https://github.com/{repository}.git"], check=True)


# id, project, checkout, branch, repository, state, task, minutes since its last change, started minutes ago, model, effort, context %
SESSIONS = [
    ("11111111-1111-4111-8111-111111111111", "acme-shop", "repos/acme-shop", "feat/checkout-vat", "acme/shop", "working",
     "Checkout totals with VAT per country", 2, 95, "Opus 5.5", "high", 41),
    ("22222222-2222-4222-8222-222222222222", "acme-shop", "home/IdeaProjects/.parallel/acme-shop/parallel-0412", "fix/cart-rounding", "acme/shop", "needs-you",
     "Cart rounding on discounted items", 0, 40, "Opus 5.5", "high", 18),
    ("33333333-3333-4333-8333-333333333333", "notes-api", "repos/notes-api", "main", "acme/notes-api", "waiting",
     "Paginate the search endpoint", 6, 70, "Sonnet 5.5", "medium", 63),
    ("44444444-4444-4444-8444-444444444444", "homelab", "repos/homelab", "monitoring", "acme/homelab", "waiting",
     "Grafana dashboard for the NAS disks", 25, 180, "Opus 5.5", "xhigh", 92),
]
VIEWED = SESSIONS[0][0]


def build(root: str):
    home, run, state = (os.path.join(root, name) for name in ("home", "run", "state"))
    sessions = []
    for (session_id, project, relative, branch, repository, state_name, task, idle_minutes, started_minutes,
         model, effort, context) in SESSIONS:
        cwd = os.path.join(root, relative)
        checkout(cwd, branch, repository)
        entry = {"session": session_id, "state": state_name, "project": project, "cwd": cwd, "task": task,
                 "since": int(NOW - idle_minutes * 60), "started": int(NOW - started_minutes * 60), "tmux_pane": f"%9{len(sessions)}"}
        if state_name == "needs-you":
            entry["request"] = "Claude needs your permission to use Bash"
            entry["detail"] = "npm test -- cart/rounding"
        if state_name == "waiting" and project == "notes-api":
            entry["outcome"] = "Finished · search pages by 50 with a cursor; 14 tests pass"
        sessions.append(entry)
        write(os.path.join(home, ".cache/claude", f"statusline-{session_id}.json"),
              json.dumps({"model": model, "effort": effort, "context": context}))
    write(os.path.join(run, "claude-sessions.json"), json.dumps({"sessions": sessions}))
    write(os.path.join(run, "claude-sessions.current"), VIEWED + "\n")
    write(os.path.join(run, "claude-usage.json"), json.dumps({
        "five_hour": {"utilization": 34, "resets_at": iso(2 * 3600 + 40 * 60)},
        "seven_day": {"utilization": 58, "resets_at": iso(3 * 86400)},
        "seven_day_opus": {"utilization": 41, "resets_at": iso(3 * 86400)}}))
    # The plan's name only: no token of any kind.
    write(os.path.join(home, ".claude/.credentials.json"), json.dumps({"claudeAiOauth": {"subscriptionType": "max"}}))
    history = [{"sessionId": f"5555555{index}-5555-4555-8555-555555555555", "project": os.path.join(root, project),
                "timestamp": int((NOW - hours * 3600) * 1000), "display": prompt}
               for index, (project, hours, prompt) in enumerate([
                   ("repos/acme-shop", 20, "Move the invoice PDF to the new template"),
                   ("repos/notes-api", 30, "Rate limit the public API per token"),
                   ("repos/homelab", 50, "Back up the Postgres volumes to the NAS nightly")])]
    write(os.path.join(home, ".claude/history.jsonl"), "".join(json.dumps(row) + "\n" for row in history))
    shop = os.path.join(root, "repos/acme-shop")
    write(os.path.join(state, "claude-sessions/todo.json"), json.dumps({"items": [
        {"id": "a1b2c3d4", "project": "acme-shop", "cwd": shop, "text": "Show prices with and without VAT on the product page",
         "status": "doing", "priority": 4, "created": int(NOW - 3600)},
        {"id": "b2c3d4e5", "project": "acme-shop", "cwd": shop, "text": "Remove the old coupon endpoint",
         "status": "todo", "priority": 2, "created": int(NOW - 7200)},
        {"id": "c3d4e5f6", "project": "acme-shop", "cwd": shop, "text": "Translate the checkout emails",
         "status": "done", "priority": 2, "created": int(NOW - 86400)}],
        "issues": {"acme/shop#57": {"status": "doing", "updated": int(NOW - 600), "session": VIEWED}}}))
    pulls = [
        {"number": 61, "title": "Checkout totals with VAT per country", "url": "https://github.com/acme/shop/pull/61",
         "updatedAt": iso(-1200), "author": {"login": "claude"}, "isDraft": True, "reviewDecision": "",
         "statusCheckRollup": [{"status": "IN_PROGRESS", "conclusion": ""}], "headRefName": "feat/checkout-vat"},
        {"number": 58, "title": "Cache product images at the edge", "url": "https://github.com/acme/shop/pull/58",
         "updatedAt": iso(-7200), "author": {"login": "sam"}, "isDraft": False, "reviewDecision": "REVIEW_REQUIRED",
         "statusCheckRollup": [{"status": "COMPLETED", "conclusion": "SUCCESS"}], "headRefName": "perf/image-cache"},
        {"number": 55, "title": "Upgrade the payment SDK", "url": "https://github.com/acme/shop/pull/55",
         "updatedAt": iso(-86400), "author": {"login": "alex"}, "isDraft": False, "reviewDecision": "CHANGES_REQUESTED",
         "statusCheckRollup": [{"status": "COMPLETED", "conclusion": "FAILURE"}], "headRefName": "deps/payment-sdk"}]
    issues = [
        {"number": 57, "title": "VAT missing on shipping costs", "url": "https://github.com/acme/shop/issues/57",
         "updatedAt": iso(-3600), "labels": [{"name": "bug"}, {"name": "P1"}]},
        {"number": 49, "title": "Wishlist export to CSV", "url": "https://github.com/acme/shop/issues/49",
         "updatedAt": iso(-5 * 86400), "labels": [{"name": "enhancement"}]}]
    write(os.path.join(root, "gh-pulls.json"), json.dumps(pulls))
    write(os.path.join(root, "gh-issues.json"), json.dumps(issues))
    write(os.path.join(root, "bin/gh"), f"""#!/bin/sh
# The demo's GitHub: only acme/shop has pull requests and issues.
case "$*" in
  "pr list --repo acme/shop"*) cat {root}/gh-pulls.json ;;
  "issue list --repo acme/shop"*) cat {root}/gh-issues.json ;;
  *) echo "[]" ;;
esac
""", executable=True)
    write(os.path.join(root, "bin/claude-sessions"), "#!/bin/sh\n# The demo has no real sessions to act on.\nexit 0\n", executable=True)
    write(os.path.join(root, "status.json"), json.dumps({"status": {"indicator": "none", "description": "All Systems Operational"},
                                                         "components": [], "incidents": []}))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: build.py <root>")
    build(os.path.abspath(sys.argv[1]))
