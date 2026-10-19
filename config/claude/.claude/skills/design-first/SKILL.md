---
name: design-first
description: For developer sessions in a project that has FigCraft designs (*.figcraft files; the session start says so). Use before implementing or changing any user interface there — a screen, a component, a flow, a state, a visual tweak — and when a task touches UI that has no design yet. Checks the design exists and holds together before coding, reports gaps, and asks for a design first instead of improvising one.
---

# Design first

This project designs its interface in FigCraft before building it. You are the developer: you build
what was designed, and you are the first to notice what the design forgot. An interface nobody
thought through, neither as UI/UX nor as code, is not something to improvise in a pull request.

## 1. Find the design before writing UI code

1. Locate the design: `git ls-files '*.figcraft'`, the project's design docs, the Notes page of the
   file (the designer sessions keep the brief, decisions and open questions there).
2. Read it through the `figcraft-app` tools (`inspect_document`, `get_node`, `screenshot`). If they
   are not connected, ask Theo to connect FigCraft as CLAUDE.md describes (`craft-open figcraft`).
   Never edit the design file from a developer task: that is the designer's work (the
   `figcraft-design` skill), and only when Theo asks.
3. Map what you will build to the design: its frames, components and variants, variables (colours,
   spacing, radii, type) and states.

## 2. No design: ask, don't improvise

When the screen, component, flow or state you are asked to build has no design (or only part of
it), stop before coding and ask with AskUserQuestion, recommendation first:

- **Design it first** (recommended for anything a user sees beyond a trivial tweak): a FigCraft
  session wireframes it with Theo, then you build it.
- **Build a minimal version from the existing system**: only existing components and tokens, no
  new visual language, and the gap filed as a design to-do (`claude-todo propose`).
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
blocking ones; file the rest with `claude-todo propose '<title>' --why '<why>'` so the designer
session picks them up.

## 4. Think the code through too

A design is half the plan. Before the first component, state briefly: which existing components
you reuse and which you create, where the state and data live, how the design's variables map to
the code's tokens (theme, CSS variables, Tailwind config), and what you will test. If the code side
raises a question (a new API field, a shared component to change, a migration), ask before building.

## 5. Build it faithfully

- Use the project's components and tokens matching the design's components and variables; never
  hard-code a value the design binds to a variable. A token missing in code is added to the theme,
  not inlined.
- Implement every designed state and breakpoint, not only the default.
- Compare the result with the design (`screenshot` of the frame next to a screenshot of the running
  page when the project can run) and list any difference you chose or could not avoid.
