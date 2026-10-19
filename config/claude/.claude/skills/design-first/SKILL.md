---
name: design-first
description: For developer sessions in a project that has FigCraft designs (*.figcraft files; the session start says so). Use before implementing or changing any user interface there — a screen, a component, a flow, a state, a visual tweak — and when a task touches UI that has no design yet. Checks the design exists and holds together before coding, reports gaps, and asks for a design first instead of improvising one. Builds components first, each in Storybook, and pages only from components.
---

# Design first

This project designs its interface in FigCraft before building it. You are the developer: you build
what was designed, and you are the first to notice what the design forgot. An interface nobody
thought through, neither as UI/UX nor as code, is not something to improvise in a pull request.

At the start, run `claude-design role developer`: Theo's cockpit shows the role on your card and
keeps the design to-dos for the designer session.

## 1. Find the design before writing UI code

1. Read `DESIGN.md` at the repository root first: the brief, decisions, components, the frames
   ready for dev and the open questions, kept by the designer sessions. Then the `.figcraft` files
   (`git ls-files '*.figcraft'`) for the frames themselves.
2. Read them through the `figcraft-app` tools: `inspect.readyForDev` (`run_command`) lists the
   frames ready for dev, then `inspect_document`, `get_node`, `screenshot`. If the tools are not
   connected, ask Theo to connect FigCraft as CLAUDE.md describes (`craft-open figcraft`).
3. **Build only frames marked ready for dev.** A frame that exists but is not marked is a draft:
   treat it as "no design" (section 2). A ready frame you or someone built before: run
   `dev.compare` on it and build what changed.
4. Never edit the design file from a developer task: that is the designer's work. The one
   exception: once a frame is built and checked against the design, set its `devStatus` to
   `completed` (`node.set`), and nothing else.
5. Map what you will build to the design: its frames, components and variants, variables (colours,
   spacing, radii, type) and states.

## 2. No design: ask, don't improvise

When the screen, component, flow or state you are asked to build has no design (or only part of
it), stop before coding and ask with AskUserQuestion, recommendation first:

- **Design it first** (recommended for anything a user sees beyond a trivial tweak): a FigCraft
  session wireframes it with Theo, then you build it.
- **Build a minimal version from the existing system**: only existing components and tokens, no
  new visual language, and the gap filed for the designer (`claude-todo add '<what to design>'
  --design`).
- **Build without a design**: Theo decides it is fine; say what you will assume.

A "trivial tweak" is one that reuses existing components and tokens unchanged (a copy fix, a field
added to a form that already has the same kind of field). When unsure, ask.

## 3. Check the design holds together, and report what doesn't

Before and while implementing, look for what the design leaves out or gets wrong, and report it to
Theo instead of patching it silently:

- **Missing states**: empty, loading, error, offline, permission denied, long content, 0/1/many
  items, the first-run case.
- **Missing sizes**: breakpoints, small screens, very wide screens, zoom and large text.
- **Inconsistencies**: raw values where the system has a token, a component redrawn instead of
  reused, two designs of the same thing, spacing off the scale.
- **Accessibility**: contrast, focus order and focus states, target sizes, labels, colour alone
  carrying meaning, keyboard and screen reader paths.
- **Things that don't make sense in code**: data the API does not provide or that is slow to get,
  interactions the platform or the stack can't do well, layouts that fight the component library,
  performance traps (huge lists, heavy effects), copy that will not fit once translated.
- **Product questions**: a flow with a dead end, an action without confirmation or undo, an
  unclear primary action.

Report them grouped, most blocking first, each with what you suggest. Ask before deciding the
blocking ones; file the rest for the designer with `claude-todo propose '<title>' --why '<why>'
--design`: once Theo approves them, the cockpit hands them to the project's designer session.

## 4. Think the code through too

A design is half the plan. Before the first component, state briefly: which existing components
you reuse and which you create, where the state and data live, how the design's variables map to
the code's tokens (theme, CSS variables, Tailwind config), and what you will test. If the code side
raises a question (a new API field, a shared component to change, a migration), ask before building.

## 5. Build from components, in Storybook

Work like a frontend developer on a design system: components first, each built and reviewed in
isolation, then pages assembled from them. Nothing visual is built directly in a page.

1. **Storybook is the workshop.** Every UI component is developed and reviewed in Storybook before
   any page uses it. If the project has no Storybook, stop and propose adding it with the setup
   that matches its framework (`npx storybook@latest init`, or the project's own way): it changes
   the dependency manifest and adds scripts Theo runs outside the sandbox, so ask first and say both.
2. **Bottom-up, mirroring the FigCraft file**: tokens (from `design/tokens.*`) → primitives (button,
   input, icon, text…) → composites (cards, rows, forms, navigation) → layouts and sections → pages.
   A level starts only once the level below has what it needs.
3. **One FigCraft component = one code component and its stories.** Same name as in FigCraft. Each
   variant and state of the component set is a story, its properties as args (size, variant,
   disabled, loading, error…), plus the edge cases (long text, empty, many items, right-to-left when
   the product needs it). Before writing one, look in the codebase and in Storybook for a component
   to reuse or extend: two components for one design component is a bug.
4. **Pages are compositions only.** A page or screen holds layout and data wiring, never ad-hoc
   styled markup: anything visual it needs is a component, and a new one goes through steps 2 and 3
   first. Pages get stories too, with mocked data, one per designed state (empty, loading, error,
   full).
5. **Each story is checked against the design**: the story's screenshot (or the running story) next
   to the FigCraft `screenshot` of the matching component and variant; list the differences.
   Accessibility is checked in Storybook (its a11y addon when the project has it, otherwise propose
   it) and with the `fixing-accessibility` skill. Interaction states are covered by play functions
   or the project's story tests when the project uses them.
6. **Done** for a UI task means: every component it touches has stories for all its designed
   variants and states, the page's stories render each designed state, the project's Storybook build
   and tests pass, and the frame is set `completed` in FigCraft (section 1).

## 6. Build it faithfully

- **Tokens come from the design's exports**, `design/tokens.json` and `design/tokens.css` (the
  designer writes them with `variable.export`): import the CSS variables, or generate the theme from
  the JSON. Never retype a value the design binds to a variable; a token missing from the exports is
  a gap for the designer (`--design`), not a value to inline.
- **The tokens are checked**: the project needs a check in its own test suite that fails when the
  code's tokens differ from `design/tokens.json` (a generated theme out of date, a token missing on
  one side). If the project has none, propose it once (`claude-todo propose`), and point out to
  Theo that test scripts run outside the sandbox.
- Use the project's components matching the design's components and variants.
- Implement every designed state and breakpoint, not only the default.
- Compare the result with the design (`screenshot` of the frame next to a screenshot of the running
  page when the project can run) and list any difference you chose or could not avoid.
