"""The manager's sidebar (config/bin/.local/bin/claude-sessions-sidebar): the parts that need no terminal."""

import importlib.machinery
import importlib.util
import json
import os
import time
import unittest

from support import BIN, SESSION_A, SESSION_B, Sandbox


def load_sidebar(sandbox: Sandbox):
    for name, value in sandbox.env.items():
        os.environ[name] = value
    os.environ["CLAUDE_SESSIONS_HISTORY"] = str(sandbox.root / "history.jsonl")
    os.environ["CLAUDE_SESSIONS_TRANSCRIPTS"] = str(sandbox.root / "transcripts")
    loader = importlib.machinery.SourceFileLoader("sidebar", str(BIN / "claude-sessions-sidebar"))
    module = importlib.util.module_from_spec(importlib.util.spec_from_loader("sidebar", loader))
    loader.exec_module(module)
    return module


class HistoryTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.sidebar = load_sidebar(self)
        now = int(time.time())
        rows = [
            {"display": "first alpha task today", "project": "/x/alpha", "sessionId": SESSION_A, "timestamp": (now - 3600) * 1000},
            {"display": "/resume", "project": "/x/alpha", "sessionId": SESSION_A, "timestamp": (now - 3500) * 1000},
            {"display": "[Pasted text #1 +9 lines]", "project": "/x/alpha", "sessionId": SESSION_A, "timestamp": (now - 3400) * 1000},
            {"display": "f", "project": "/x/beta", "sessionId": SESSION_B, "timestamp": (now - 100000) * 1000},
        ]
        (self.root / "history.jsonl").write_text("\n".join(json.dumps(row) for row in rows) + "\n")
        transcripts = self.root / "transcripts/-x-alpha"
        transcripts.mkdir(parents=True)
        (transcripts / f"{SESSION_A}.jsonl").write_text(json.dumps({"type": "ai-title", "aiTitle": "Alpha work", "sessionId": SESSION_A}) + "\n")
        (self.root / "transcripts/-x-beta").mkdir()
        (self.root / "transcripts/-x-beta" / f"{SESSION_B}.jsonl").write_text("")
        self.history = self.sidebar.History()
        self.history.refresh()
        self.view = self.sidebar.ViewState()

    def test_latest_real_prompt_wins_over_commands_and_pastes(self):
        entry = next(entry for entry in self.history.entries if entry["session"] == SESSION_A)
        self.assertEqual(entry["prompt"], "first alpha task today")

    def test_generated_title_replaces_the_prompt(self):
        self.history.load_titles([SESSION_A], 5)
        row = next(row for row in self.history.recent(set(), "", self.view) if row["session"] == SESSION_A)
        self.assertEqual(row["task"], "Alpha work")

    def test_trivial_sessions_hide_until_asked_for(self):
        self.history.load_titles([SESSION_A, SESSION_B], 5)
        self.assertNotIn(SESSION_B, [row["session"] for row in self.history.recent(set(), "", self.view)])
        self.view.show_trivial = True
        self.assertIn(SESSION_B, [row["session"] for row in self.history.recent(set(), "", self.view)])

    def test_running_sessions_and_filter_narrow_the_list(self):
        self.history.load_titles([SESSION_A, SESSION_B], 5)
        self.assertEqual([row["session"] for row in self.history.recent({SESSION_A}, "", self.view)], [])
        self.assertEqual([row["session"] for row in self.history.recent(set(), "alpha first", self.view)], [SESSION_A])


class LayoutTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.sidebar = load_sidebar(self)

    def test_fit_cuts_with_an_ellipsis(self):
        self.assertEqual(self.sidebar.fit("a very long title that goes on", 12), "a very long…")
        self.assertEqual(self.sidebar.fit("short", 12), "short")

    def test_day_buckets_and_clock(self):
        now = int(time.time())
        self.assertEqual(self.sidebar.day_bucket(now - 60), "TODAY")
        self.assertEqual(self.sidebar.day_bucket(now - 3 * 86400), "THIS WEEK")
        self.assertEqual(self.sidebar.day_bucket(now - 30 * 86400), "OLDER")

    def test_focused_panel_takes_half_the_column_after_the_fixed_status(self):
        self.sidebar.style = lambda *arguments, **keywords: 0
        app = self.sidebar.App()
        app.panels[0].build(app)  # no usage cache: one explanatory line, three with the borders
        self.assertEqual(app.layout(45), [3, 20, 10, 10])
        app.focus = 3
        self.assertEqual(app.layout(45), [3, 10, 10, 20])
        app.focus = 0
        self.assertEqual(app.layout(45), [3, 20, 10, 10])
        self.assertTrue(all(height >= 3 for height in app.layout(12)))

    def test_tab_digits_and_function_keys_move_the_focus(self):
        app = self.sidebar.App()
        self.assertEqual(app.focus, 1)
        app.handle(self.sidebar.KEY_TAB, None)
        self.assertEqual(app.focus, 2)
        app.handle(ord("0"), None)
        self.assertEqual(app.focus, 0)
        app.handle(self.sidebar.curses.KEY_BTAB, None)
        self.assertEqual(app.focus, 3)
        app.handle(ord("/"), None)
        app.handle(self.sidebar.curses.KEY_F1, None)
        self.assertEqual((app.focus, app.filtering), (1, False))

    def test_status_lists_the_usage_windows_like_the_website(self):
        self.sidebar.style = lambda *arguments, **keywords: 0
        soon = (self.sidebar.datetime.now(self.sidebar.timezone.utc) + self.sidebar.timedelta(hours=2)).strftime("%Y-%m-%dT%H:%M:%SZ")
        (self.runtime / "claude-usage.json").write_text(json.dumps({
            "five_hour": {"utilization": 59.0, "resets_at": soon},
            "limits": [
                {"kind": "session", "percent": 59, "resets_at": soon, "scope": None},
                {"kind": "weekly_all", "percent": 19, "resets_at": "2026-10-23T23:59:59.979936+00:00", "scope": None},
                {"kind": "weekly_scoped", "percent": 34, "resets_at": "2026-10-23T23:59:59.980078+00:00",
                 "scope": {"model": {"id": None, "display_name": "Fable"}, "surface": None}}],
            "seven_day_breakdown": {"rows": [{"key": "claude_code", "display_name": "Claude Code", "percent": 72},
                                             {"key": "chat", "display_name": "Chats", "percent": 0}]},
            "spend": {"used": {"amount_minor": 0, "currency": "EUR", "exponent": 2},
                      "limit": {"amount_minor": 17000, "currency": "EUR", "exponent": 2}, "enabled": False}}))
        self.sidebar.USAGE_CACHE_PATH = str(self.runtime / "claude-usage.json")
        windows = self.sidebar.usage_windows()
        self.assertEqual([(w["label"], w["percentage"], w["note"]) for w in windows],
                         [("Current session", 59, ""), ("This week", 19, ""), ("Fable this week", 34, "separate weekly limit for Fable")])
        self.assertRegex(windows[0]["resets"], r"^\d\d:\d\d$")
        self.assertRegex(windows[1]["resets"], r"^\w{3} \d\d:\d\d$")
        self.assertEqual(self.sidebar.usage_breakdown(), [("Claude Code", 72)])
        self.assertEqual(self.sidebar.usage_credits(), "€0 of €170 usage credits  (off)")
        report = "\n".join(self.sidebar.usage_report_lines(80))
        self.assertIn("Fable this week", report)
        self.assertIn("This week by product", report)
        self.assertRegex(report, r"resets \w{3} \d\d:\d\d  \(in \d+d \d+h\)")

    def test_facts_are_read_however_old_the_file_is(self):
        cache = self.home / ".cache/claude"
        cache.mkdir(parents=True, exist_ok=True)
        path = cache / f"statusline-{SESSION_A}.json"
        path.write_text(json.dumps({"model": "Fable 5.1", "context": 40}))
        os.utime(path, (time.time() - 5 * 86400, time.time() - 5 * 86400))
        self.sidebar.STATUSLINE_DIR = str(cache)
        self.assertEqual(self.sidebar.statusline_facts(SESSION_A)["model"], "Fable 5.1")

    def test_an_updated_session_shows_the_glyph_and_the_restart_hint(self):
        self.sidebar.style = lambda *arguments, **keywords: 0
        (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "state": "waiting", "project": "runway", "task": "t", "since": 1, "tmux_pane": "%4", "update": True}]}))
        self.sidebar.SNAPSHOT_PATH = str(self.runtime / "claude-sessions.json")
        self.sidebar.tmux_claude_panes = lambda: []
        app = self.sidebar.App()
        app.panels[1].build(app)
        first_line = app.panels[1].lines[0]
        self.assertIn("⬆ update", [part[0] for part in first_line.right])
        self.assertIn(("r", "restart to apply the update"), app.panels[1].hints())

    def test_the_viewed_sessions_state_is_pushed_only_when_it_changes(self):
        self.sidebar.style = lambda *arguments, **keywords: 0
        (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "state": "waiting", "project": "runway", "task": "t", "since": 1, "tmux_pane": "%4"}]}))
        (self.runtime / "claude-sessions.current").write_text(SESSION_A + "\n")
        self.sidebar.SNAPSHOT_PATH = str(self.runtime / "claude-sessions.json")
        self.sidebar.CURRENT_PATH = str(self.runtime / "claude-sessions.current")
        self.sidebar.tmux_claude_panes = lambda: []
        pushed = []
        self.sidebar.manager_in_background = lambda *arguments: pushed.append(arguments)
        app = self.sidebar.App()
        app.panels[1].build(app); app.push_viewed_state()
        app.panels[1].build(app); app.push_viewed_state()
        self.assertEqual(pushed, [("label-state", "waiting")])
        (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "state": "needs-you", "project": "runway", "task": "t", "since": 1, "tmux_pane": "%4"}]}))
        app.panels[1].build(app); app.push_viewed_state()
        self.assertEqual(pushed[-1], ("label-state", "needs-you"))

    def test_git_branch_reads_head_through_worktrees_and_detached_heads(self):
        repo = self.root / "repo"
        (repo / ".git").mkdir(parents=True)
        (repo / ".git/HEAD").write_text("ref: refs/heads/feature/panels\n")
        self.assertEqual(self.sidebar.git_branch(str(repo)), "feature/panels")
        worktree = self.root / "worktree"
        worktree.mkdir()
        (repo / ".git/worktrees/wt").mkdir(parents=True)
        (repo / ".git/worktrees/wt/HEAD").write_text("0123456789abcdef\n")
        (worktree / ".git").write_text(f"gitdir: {repo}/.git/worktrees/wt\n")
        self.assertEqual(self.sidebar.git_branch(str(worktree)), "0123456")
        self.assertEqual(self.sidebar.git_branch(str(self.root)), "")
        self.assertEqual(self.sidebar.git_branch(""), "")

    def test_status_bars_fill_the_column(self):
        self.sidebar.style = lambda *arguments, **keywords: 0
        (self.runtime / "claude-usage.json").write_text(json.dumps({"limits": [
            {"kind": "session", "percent": 50, "resets_at": "2026-10-20T09:00:00Z"}]}))
        self.sidebar.USAGE_CACHE_PATH = str(self.runtime / "claude-usage.json")
        app = self.sidebar.App()
        app.width = 60
        app.panels[0].build(app)
        bar = app.panels[0].lines[0].segments[1][1]
        self.assertEqual(len(bar), 60 - 4 - len("Current session") - 2 - 5 - (len(app.panels[0].lines[0].right[0][0]) + 2))
        app.width = 200
        app.panels[0].build(app)
        self.assertEqual(len(app.panels[0].lines[0].segments[1][1]), self.sidebar.USAGE_BAR_MAX)

    def effort_app(self, state="waiting", effort="high"):
        self.sidebar.style = lambda *arguments, **keywords: 0
        (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "state": state, "project": "runway", "task": "t", "since": 1, "tmux_pane": "%4"}]}))
        self.sidebar.SNAPSHOT_PATH = str(self.runtime / "claude-sessions.json")
        self.sidebar.tmux_claude_panes = lambda: []
        self.sidebar.statusline_facts = lambda session_id: {"model": "Opus 5.5", "context": 20, "effort": effort}
        restarts = []
        self.sidebar.manager_in_background = lambda *arguments: restarts.append(arguments)
        app = self.sidebar.App()
        app.focus = 1
        app.panels[1].build(app)
        return app, restarts

    def test_the_card_shows_the_effort_as_a_clickable_stamp(self):
        app, _ = self.effort_app()
        detail = app.panels[1].lines[1]
        self.assertEqual([part[0] for part in detail.right], ["Opus 5.5", "high ▾", "20%"])
        self.assertEqual(detail.right[1][2], "effort")

    def test_cards_keep_their_place_when_states_change(self):
        self.sidebar.style = lambda *arguments, **keywords: 0
        self.sidebar.SNAPSHOT_PATH = str(self.runtime / "claude-sessions.json")
        self.sidebar.tmux_claude_panes = lambda: []
        self.sidebar.statusline_facts = lambda session_id: {}
        def order(rows):
            (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": rows}))
            app = self.sidebar.App()
            app.panels[1].build(app)
            return [session["project"] for session in app.panels[1].items]
        first = {"session": SESSION_A, "state": "waiting", "project": "dotfiles", "since": 50, "started": 10}
        second = {"session": SESSION_B, "state": "working", "project": "runway", "since": 60, "started": 20}
        self.assertEqual(order([second, first]), ["dotfiles", "runway"])
        self.assertEqual(order([{**second, "state": "needs-you"}, first]), ["dotfiles", "runway"])

    def test_internal_messages_are_not_shown_as_tasks(self):
        self.assertEqual(self.sidebar.visible_task({"task": '<agent-message from="a05b">'}), "")
        self.assertEqual(self.sidebar.visible_task({"task": "Fix login"}), "Fix login")

    def test_choosing_another_effort_restarts_the_session_on_it(self):
        app, restarts = self.effort_app()
        app.handle(ord("e"), None)
        self.assertEqual(app.menu.current, "high")
        app.handle(self.sidebar.curses.KEY_DOWN, None)
        app.handle(self.sidebar.KEY_ENTER_LF, None)
        self.assertIsNone(app.menu)
        self.assertEqual(restarts, [("restart", SESSION_A, "xhigh")])

    def test_the_current_effort_or_escape_restarts_nothing(self):
        app, restarts = self.effort_app()
        app.handle(ord("e"), None)
        app.handle(self.sidebar.KEY_ENTER_LF, None)
        app.handle(ord("e"), None)
        app.handle(self.sidebar.KEY_ESCAPE, None)
        self.assertEqual(restarts, [])
        self.assertIsNone(app.menu)

    def test_a_digit_picks_a_level_directly(self):
        app, restarts = self.effort_app()
        app.handle(ord("e"), None)
        app.handle(ord("5"), None)
        self.assertEqual(restarts, [("restart", SESSION_A, "max")])

    def transcript(self, name, *replies):
        folder = self.root / "transcripts" / "-proj"
        folder.mkdir(parents=True, exist_ok=True)
        lines = [json.dumps({"type": "assistant", "requestId": request, "timestamp": stamp,
                             "message": {"id": message, "model": model, "usage": usage}})
                 for message, request, stamp, model, usage in replies]
        (folder / name).write_text("\n".join(lines) + "\n")
        return str(self.root / "transcripts")

    def test_reply_cost_prices_every_token_kind(self):
        usage = {"input_tokens": 1_000_000, "output_tokens": 1_000_000, "cache_read_input_tokens": 1_000_000,
                 "cache_creation_input_tokens": 2_000_000,
                 "cache_creation": {"ephemeral_5m_input_tokens": 1_000_000, "ephemeral_1h_input_tokens": 1_000_000}}
        self.assertAlmostEqual(self.sidebar.reply_cost("claude-opus-5-5", usage), 4 + 20 + 0.2 + 5 + 8)
        self.assertAlmostEqual(self.sidebar.reply_cost("claude-opus-5-5", {**usage, "speed": "fast"}), 2 * (4 + 20 + 0.2 + 5 + 8))
        self.assertAlmostEqual(self.sidebar.reply_cost("claude-haiku-4-5-20251001", {"output_tokens": 1_000_000}), 5)
        self.assertIsNone(self.sidebar.reply_cost("claude-unknown-9", usage))

    def test_haiku_5_5_switches_tier_above_100k_prompt_tokens(self):
        self.assertAlmostEqual(self.sidebar.reply_cost("claude-haiku-5-5", {"input_tokens": 50_000}), 0.005)
        self.assertAlmostEqual(self.sidebar.reply_cost("claude-haiku-5-5", {"input_tokens": 200_000}), 0.1)

    def test_a_reply_copied_into_a_forked_transcript_counts_once(self):
        stamp = self.sidebar.datetime.now(self.sidebar.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.000Z")
        reply = ("msg_1", "req_1", stamp, "claude-opus-5-5", {"output_tokens": 1_000_000})
        self.transcript("a.jsonl", reply, ("msg_2", "req_2", stamp, "claude-opus-5-5", {"output_tokens": 500_000}))
        directory = self.transcript("b.jsonl", reply, ("msg_x", "req_x", stamp, "<synthetic>", {"output_tokens": 9}),
                                    ("msg_3", "req_3", stamp, "claude-new-model", {"output_tokens": 7}))
        days, unpriced = self.sidebar.transcript_costs(directory, str(self.state / "usage.sqlite"))
        self.assertAlmostEqual(days[self.sidebar.date.today().isoformat()], 30)
        self.assertEqual(unpriced, {"claude-new-model": 7})

    def test_the_archive_keeps_replies_after_their_transcript_is_deleted(self):
        stamp = "2026-09-20T10:00:00.000Z"
        directory = self.transcript("old.jsonl", ("msg_1", "req_1", stamp, "claude-opus-5-5", {"output_tokens": 1_000_000}))
        archive_path = str(self.state / "usage.sqlite")
        self.sidebar.transcript_costs(directory, archive_path)
        (self.root / "transcripts/-proj/old.jsonl").unlink()
        days, _ = self.sidebar.transcript_costs(directory, archive_path)
        self.assertAlmostEqual(sum(days.values()), 20)
        archive = self.sidebar.open_archive(archive_path)
        self.assertEqual(archive.execute("select project, model, output from replies").fetchall(), [("?", "claude-opus-5-5", 1_000_000)])
        self.assertAlmostEqual(self.sidebar.archived_costs(archive)[2], 20)
        archive.close()

    def test_unchanged_transcripts_are_not_read_again(self):
        stamp = "2026-10-01T10:00:00.000Z"
        directory = self.transcript("a.jsonl", ("msg_1", "req_1", stamp, "claude-opus-5-5", {"output_tokens": 10}))
        archive = self.sidebar.open_archive(str(self.state / "usage.sqlite"))
        self.assertEqual(self.sidebar.archive_transcripts(archive, directory), 1)
        reads = []
        real_open = open
        self.sidebar.open = lambda *arguments, **keywords: reads.append(arguments[0]) or real_open(*arguments, **keywords)
        self.assertEqual(self.sidebar.archive_transcripts(archive, directory), 0)
        self.assertEqual(reads, [])
        archive.close()

    def test_week_and_month_add_up_the_days(self):
        costs = self.sidebar.TranscriptCosts()
        today = self.sidebar.date.today()
        costs.days = {today.isoformat(): 10, (today - self.sidebar.timedelta(days=today.weekday())).isoformat(): 5,
                      (today - self.sidebar.timedelta(days=40)).isoformat(): 1000}
        totals = costs.totals()
        self.assertEqual(totals["today"], 10)
        self.assertEqual(totals["week"], 15 if today.weekday() else 10)
        self.assertLess(totals["month"], 1000)
        self.assertEqual((self.sidebar.money(112.5), self.sidebar.money(9.813), self.sidebar.money(12345)), ("$112", "$9.81", "$12.3k"))

    def test_budget_alerts_fire_once_per_threshold_and_window(self):
        self.sidebar.ALERTS_PATH = str(self.state / "alerts.json")
        sent = []
        alerts = self.sidebar.BudgetAlerts(send=lambda notice: sent.append(notice) or True)
        week = {"key": "weekly_all", "label": "This week", "percentage": 82, "resets_at": "2026-10-23T23:59:59Z", "resets": "Sat 01:59"}
        alerts.check([week])
        alerts.check([week])
        self.assertEqual([notice["title"] for notice in sent], ["Claude · This week at 82%"])
        alerts.check([{**week, "percentage": 96}])
        self.assertEqual(len(sent), 2)
        again = self.sidebar.BudgetAlerts(send=lambda notice: sent.append(notice) or True)
        again.check([{**week, "percentage": 96}])
        self.assertEqual(len(sent), 2)  # remembered across a restart
        again.check([{**week, "percentage": 85, "resets_at": "2026-10-30T23:59:59Z"}])
        self.assertEqual(len(sent), 3)  # a new window alerts again

    def test_a_full_context_suggests_compact_on_the_card(self):
        app, _ = self.effort_app()
        self.sidebar.statusline_facts = lambda session_id: {"model": "Opus 5.5", "context": 93, "effort": "high"}
        app.panels[1].build(app)
        self.assertIn("/compact", [part[0] for part in app.panels[1].lines[0].right])

    def test_the_picker_lists_recent_projects_first(self):
        projects = self.root / "IdeaProjects"
        for name in ("alpha", "beta", "gamma"):
            (projects / name).mkdir(parents=True)
        history = self.root / "picker-history.jsonl"
        history.write_text("\n".join(json.dumps(entry) for entry in (
            {"project": str(projects / "gamma"), "timestamp": 100},
            {"project": str(projects / "beta"), "timestamp": 300},
            {"project": str(projects / "gamma"), "timestamp": 200},
            {"project": str(projects / "gone"), "timestamp": 400},
            {"project": str(self.root), "timestamp": 500},
            {"project": str(projects / "alpha" / "src"), "timestamp": 250})) + "\n")
        self.sidebar.PROJECT_PATTERNS = (str(projects) + "/*/",)
        self.assertEqual([os.path.basename(path) for path in self.sidebar.project_choices(str(history))], ["beta", "alpha", "gamma"])

    def test_a_title_written_later_is_picked_up(self):
        history = self.sidebar.History()
        titles = {SESSION_A: ""}
        self.sidebar.read_title = lambda session_id: titles[session_id]
        history.load_titles([SESSION_A], 8)
        titles[SESSION_A] = "Effort menu"
        history.load_titles([SESSION_A], 8)
        self.assertEqual(history.title(SESSION_A), "")  # not re-read within TITLE_RETRY_S
        history.title_checked[SESSION_A] -= self.sidebar.TITLE_RETRY_S
        history.load_titles([SESSION_A], 8)
        self.assertEqual(history.title(SESSION_A), "Effort menu")

    def test_the_digest_covers_the_last_day_today_waiting_and_pull_requests(self):
        archive = self.sidebar.open_archive(str(self.state / "usage.sqlite"))
        today = self.sidebar.date(2026, 10, 19)  # a Monday: the last day with work is Saturday
        rows = [("k1", "2026-10-17", "t", "claude-opus-5-5", "runway", SESSION_A, 0, 1000, 0, 0, 0, "standard", 12.5),
                ("k2", "2026-10-17", "t", "claude-opus-5-5", "runway", SESSION_A, 0, 1000, 0, 0, 0, "standard", 2.0),
                ("k2b", "2026-10-17", "t", "claude-opus-5-5", "runway", SESSION_A, 0, 1000, 0, 0, 0, "standard", 0.5),
                ("k2c", "2026-10-17", "t", "claude-opus-5-5", "api", "cccccccc-0000-4000-8000-000000000000", 0, 10, 0, 0, 0, "standard", 0.25),
                ("k3", "2026-10-19", "t", "claude-opus-5-5", "dotfiles", SESSION_B, 0, 1000, 0, 0, 0, "standard", 4.0)]
        archive.executemany("insert into replies values (?,?,?,?,?,?,?,?,?,?,?,?,?)", rows)
        self.sidebar.read_title = lambda session_id: {SESSION_A: "Runway project setup"}.get(session_id, "")
        self.sidebar.usage_windows = lambda: []
        sessions = [{"session": SESSION_B, "state": "needs-you", "project": "dotfiles", "task": "Effort menu"},
                    {"session": SESSION_A, "state": "working", "project": "runway", "task": "x"}]
        reviews = [{"repository": "runway", "number": 7, "title": "Fix", "ci": "passing", "review": ""},
                   {"repository": "runway", "number": 8, "title": "Break", "ci": "failing", "review": "CHANGES_REQUESTED"}]
        text = self.sidebar.re.sub(r"\x1b\[[0-9;]*m", "", "\n".join(self.sidebar.digest_lines(archive, sessions, reviews, today)))
        self.assertIn("Saturday 17 Oct  $15.25", text)
        self.assertIn("Runway project setup", text)
        self.assertIn("and 1 short session · $0.25", text)
        self.assertIn("Today so far  $4.00", text)
        self.assertIn("! dotfiles", text)
        self.assertNotIn("○ runway", text)  # working, not waiting
        self.assertLess(text.index("runway#8"), text.index("runway#7"))  # failing CI first
        archive.close()

    def test_the_sidebar_logs_on_one_line_in_the_shared_format(self):
        self.sidebar.LOG_PATH = str(self.state / "claude-sessions/log")
        self.sidebar.log_event("ERROR", "crashed: line one\n  line two")
        line = (self.state / "claude-sessions/log").read_text().splitlines()[0]
        self.assertRegex(line, r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d[+-]\d\d:\d\d ERROR sidebar: crashed: line one line two$")

    def test_logging_never_raises_on_a_read_only_state_dir(self):
        self.sidebar.LOG_PATH = "/proc/claude-sessions/log"
        self.sidebar.log_event("INFO", "nowhere to go")

    def test_a_card_needing_you_says_what_it_asks(self):
        self.assertEqual(self.sidebar.waiting_request({"state": "needs-you", "request": "Claude needs your permission to use Bash",
                                                       "detail": "Bash command · npm test"}), "Bash command · npm test")
        self.assertEqual(self.sidebar.waiting_request({"state": "needs-you", "request": "Claude has a question"}), "Claude has a question")
        self.assertEqual(self.sidebar.waiting_request({"state": "working", "request": "stale"}), "")

    def test_only_the_sessions_alive_at_shutdown_are_offered_back(self):
        boot = 100_000
        recent = {
            "alive-1": {"project": "runway", "cwd": "/r", "last_seen": boot - 120},
            "alive-2": {"project": "opusline", "cwd": "/o", "last_seen": boot - 400},
            "closed-earlier": {"project": "api", "cwd": "/a", "last_seen": boot - 5000},
            "exited": {"project": "web", "cwd": "/w", "last_seen": boot - 60},
            "nothing-said": {"project": "homelab", "cwd": "/h", "last_seen": boot - 60},
            "already-back": {"project": "dotfiles", "cwd": "/d", "last_seen": boot - 60},
            "after-boot": {"project": "new", "cwd": "/n", "last_seen": boot + 60},
        }
        offer = self.sidebar.sessions_to_restore(recent, {"exited"}, {"already-back"}, boot,
                                                 lambda session: session != "nothing-said")
        self.assertEqual([row["session"] for row in offer], ["alive-2", "alive-1"])  # by project

    def test_nothing_is_offered_without_sessions_from_before_the_boot(self):
        self.assertEqual(self.sidebar.sessions_to_restore({"s": {"cwd": "/x", "last_seen": 200}}, set(), set(), 100, lambda s: True), [])

    def test_focus_keeps_only_the_sessions_waiting_on_you(self):
        self.sidebar.style = lambda *arguments, **keywords: 0
        self.sidebar.SNAPSHOT_PATH = str(self.runtime / "claude-sessions.json")
        self.sidebar.tmux_claude_panes = lambda: []
        self.sidebar.statusline_facts = lambda session_id: {}
        (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "state": "working", "project": "runway", "since": 1, "started": 1},
            {"session": SESSION_B, "state": "needs-you", "project": "opusline", "since": 1, "started": 2}]}))
        app = self.sidebar.App()
        app.focus = 1
        app.handle(ord("f"), None)
        app.panels[1].build(app)
        self.assertEqual([session["project"] for session in app.panels[1].items], ["opusline"])
        app.handle(ord("f"), None)
        app.panels[1].build(app)
        self.assertEqual(len(app.panels[1].items), 2)

    def test_the_palette_lists_one_key_actions_and_runs_them_in_their_panel(self):
        app = self.sidebar.App()
        entries = self.sidebar.palette_entries(app)
        keys = {(panel, key) for panel, key, _ in entries}
        self.assertIn((1, "e"), keys)
        self.assertIn((None, "a"), keys)
        self.assertFalse(any(key in ("j/k ↑↓", "0-3", ":") for _, key, _ in entries))
        app.focus = 3
        self.sidebar.run_palette_entry(app, (1, "f", "focus"), None)
        self.assertEqual((app.focus, app.focus_waiting), (1, True))

    def test_greyproxy_requests_retried_lately_count_as_held(self):
        path = self.root / "greyproxy.db"
        database = self.sidebar.sqlite3.connect(path)
        database.execute("create table pending_requests (container_name text, destination_host text, destination_port integer, "
                         "first_seen datetime, last_seen datetime, attempt_count integer)")
        database.execute("insert into pending_requests values ('c', 'api.stripe.com', 443, datetime('now'), datetime('now', '-2 minutes'), 3)")
        database.execute("insert into pending_requests values ('c', 'old.example', 80, datetime('now', '-3 days'), datetime('now', '-3 days'), 1)")
        database.commit()
        database.close()
        self.assertEqual(self.sidebar.greyproxy_held(str(path)), ["api.stripe.com:443"])
        self.assertEqual(self.sidebar.greyproxy_held(str(self.root / "missing.db")), [])

    def test_status_falls_back_to_the_flat_windows_without_a_limits_list(self):
        (self.runtime / "claude-usage.json").write_text(json.dumps({
            "five_hour": {"utilization": 37.4, "resets_at": "2026-10-20T09:00:00Z"}, "seven_day": {"utilization": 72},
            "seven_day_opus": None, "seven_day_fable": {"utilization": 12}, "model_scoped": [{"display_name": "Sonnet", "utilization": 5}]}))
        self.sidebar.USAGE_CACHE_PATH = str(self.runtime / "claude-usage.json")
        self.assertEqual([(w["label"], w["percentage"]) for w in self.sidebar.usage_windows()],
                         [("Current session", 37), ("This week", 72), ("Fable this week", 12), ("Sonnet this week", 5)])
        self.assertEqual(self.sidebar.usage_bar(50, 12), "██████░░░░░░")
        self.assertEqual(self.sidebar.usage_bar(None, 12), "············")


class ReviewsTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.fixture = self.root / "reviews.json"
        os.environ["CLAUDE_REVIEWS_FIXTURE"] = str(self.fixture)
        self.sidebar = load_sidebar(self)

    def tearDown(self):
        os.environ.pop("CLAUDE_REVIEWS_FIXTURE", None)
        super().tearDown()

    def test_parses_gh_output_into_rows(self):
        self.fixture.write_text(json.dumps([{"url": "https://github.com/o/r/pull/7", "title": "Fix the thing",
                                             "repository": {"nameWithOwner": "o/r"}, "updatedAt": "2026-10-17T01:00:00Z",
                                             "author": {"login": "bob"}, "number": 7}]))
        rows, error = self.sidebar.fetch_reviews([])
        self.assertEqual(error, "")
        self.assertEqual((rows[0]["repository"], rows[0]["number"], rows[0]["author"]), ("r", 7, "bob"))

    def test_query_follows_the_running_sessions_repositories(self):
        listed = self.sidebar.gh_list_command("Androlax2/runway")
        self.assertEqual(listed[:5], ["gh", "pr", "list", "--repo", "Androlax2/runway"])
        self.assertIn("statusCheckRollup", listed[-1])
        searched = self.sidebar.gh_command([])
        self.assertIn("--review-requested=@me", searched)
        self.assertIn("-author:app/dependabot", searched)

    def test_ci_and_review_state_come_from_gh_pr_list(self):
        raw = json.dumps([
            {"number": 7, "title": "Fix", "url": "u", "updatedAt": "2026-10-17T01:00:00Z", "author": {"login": "theo"},
             "isDraft": False, "reviewDecision": "APPROVED",
             "statusCheckRollup": [{"conclusion": "SUCCESS"}, {"status": "IN_PROGRESS", "conclusion": ""}]},
            {"number": 8, "title": "Bump", "url": "u", "updatedAt": "2026-10-17T01:00:00Z", "author": {"login": "app/dependabot"},
             "statusCheckRollup": [{"conclusion": "FAILURE"}]}])
        rows = self.sidebar.parse_reviews(raw, "Androlax2/runway")
        self.assertEqual([(row["repository"], row["ci"], row["review"]) for row in rows],
                         [("runway", "pending", "APPROVED"), ("runway", "failing", "")])
        self.assertEqual([row["number"] for row in rows if self.sidebar.wanted_review(row, "2026-01-01")], [7])
        self.assertEqual(self.sidebar.ci_state([{"conclusion": "SUCCESS"}, {"conclusion": "SKIPPED"}]), "passing")
        self.assertEqual(self.sidebar.ci_state([]), "")

    def test_repositories_come_from_the_sessions_git_origins(self):
        project = self.root / "proj"
        project.mkdir()
        self.shim("git", 'echo "git@github.com:Androlax2/runway.git"')
        (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": [{"cwd": str(project)}, {"cwd": str(project)}, {}]}))
        os.environ["PATH"] = self.env["PATH"]
        self.assertEqual(self.sidebar.session_repositories(), ["Androlax2/runway"])

    def test_the_viewed_session_narrows_the_repositories_to_its_own(self):
        first, second = self.root / "first", self.root / "second"
        first.mkdir(); second.mkdir()
        self.shim("git", 'case "$*" in *first*) echo "git@github.com:o/first.git";; *) echo "https://github.com/o/second";; esac')
        (self.runtime / "claude-sessions.json").write_text(json.dumps({"sessions": [
            {"session": SESSION_A, "cwd": str(first)}, {"session": SESSION_B, "cwd": str(second)}]}))
        os.environ["PATH"] = self.env["PATH"]
        self.assertEqual(self.sidebar.session_repositories(), ["o/first", "o/second"])
        self.assertEqual(self.sidebar.session_repositories(SESSION_B), ["o/second"])
        self.assertEqual(self.sidebar.session_repositories("not-running"), ["o/first", "o/second"])

    def test_bad_json_gives_no_rows(self):
        self.assertEqual(self.sidebar.parse_reviews("not json"), [])

    def test_pull_requests_open_in_the_cockpit_window(self):
        self.assertEqual(self.sidebar.scratchpad_call("prs", "https://github.com/o/r/pull/7"),
                         ["hyprctl", "eval", 'OPEN_IN_SCRATCHPAD("prs", "https://github.com/o/r/pull/7")'])
        self.assertEqual(self.sidebar.scratchpad_call("claude"), ["hyprctl", "eval", 'OPEN_IN_SCRATCHPAD("claude")'])
        for bad in ('https://x/"); os.exit(', "file:///etc/passwd", "https://x/\\n"):
            with self.assertRaises(ValueError):
                self.sidebar.scratchpad_call("prs", bad)


class KillTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.sidebar = load_sidebar(self)
        self.shim("claude-sessions", "")
        self.shim("hyprctl", "")
        self.shim("osd-show", "")

    def test_exit_through_the_manager_when_there_is_a_pane(self):
        self.assertIn("exiting", self.sidebar.kill_session({"session": SESSION_A, "project": "p", "tmux_pane": "%3"}))
        self.assertEqual(self.wait_for_calls("claude-sessions", 1), [f"claude-sessions exit {SESSION_A}"])

    def test_closes_the_window_without_a_pane(self):
        self.sidebar.kill_session({"session": SESSION_A, "project": "p", "kitty_pid": "4242"})
        self.assertEqual(self.calls("hyprctl"), ['hyprctl dispatch hl.dsp.window.close({ window = "pid:4242" })'])

    def test_forgets_a_session_with_nothing_to_kill(self):
        self.sidebar.kill_session({"session": SESSION_A, "project": "p"})
        self.assertIn('"state": "gone"', self.calls("osd-show")[0])


if __name__ == "__main__":
    unittest.main()
