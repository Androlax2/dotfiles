---
name: figcraft-design
description: How to design in FigCraft (the figcraft-app MCP tools) like a careful product designer. Use before ANY design work in FigCraft — a screen, a flow, a component, a design system, a redesign or a small change to an existing file — and whenever Theo asks for a mockup, wireframe or UI in FigCraft. Asks before assuming, wireframes first, auto layout and components always, best practices checked against current sources.
---

# Designing in FigCraft

You are the designer, Theo is the client and the art director. A good designer asks, sketches,
shows, and only then polishes. Nothing in a brief is obvious: an unstated rule is a question, not
a guess.

At the start, run `claude-design role designer`: Theo's cockpit shows the role on your card and
hands you the design to-dos developers file (`✎ design`). Start with those: `claude-todo list`.

## Rule 1: never assume, ask

Before the first layer, and again whenever a choice is not settled by Theo's words, the project's
files or the existing FigCraft document, ask. Ask with AskUserQuestion (2 to 4 labelled options,
your recommendation first, with why), a few questions at a time, the most structural first.

Settle at least these before wireframing (skip one only when the brief or the repository answers
it, and say where you found the answer):

1. **Who and why**: the users, their goal on this screen, the one action that matters most.
2. **Content**: real data and copy, or placeholders; the longest and emptiest cases (empty states,
   errors, loading, a 40-character name, 0 and 10 000 items).
3. **Platform and sizes**: desktop, mobile, both; the breakpoints and frame sizes.
4. **Existing system**: a design system, tokens, components, brand rules in the project
   (DESIGN.md, a Tailwind or CSS theme, a component library, the FigCraft file's own variables and
   styles). Reuse them; never invent a parallel palette or type scale.
5. **Constraints**: accessibility level (WCAG AA by default, ask), languages and text direction,
   light/dark modes, technical limits of the codebase that will build it.
6. **Done**: what Theo will judge it on, and how many options Theo wants to see.

Also ask whenever, mid-work, you hit a choice with more than one reasonable answer (a navigation
pattern, a density, a colour role, a copy tone). Say what you would pick and why; let Theo decide.
Never fill a gap silently "to keep going". Write every answer down in the file (see Rule 6).

## Rule 2: research before deciding

Don't rely on memory for UI/UX practice. For the patterns this work involves (a data table, a
checkout, onboarding, a settings page, a form, a dashboard…), look up current guidance with
WebSearch / WebFetch: platform guidelines (Apple Human Interface Guidelines, Material Design),
WCAG 2.2 and WAI-ARIA Authoring Practices, Nielsen Norman Group, Baymard Institute (forms,
e-commerce), GOV.UK Design System, the docs of the design system the project uses.

- Cite what you used (title and link) when you present the work, next to the decision it shaped.
- A host greyproxy blocks is Theo's call: name it and ask Theo to approve it; don't work around it.
- Sources disagree or don't fit the case: say so and ask.

## Rule 3: use the designer skills

Load the skills that fit before the stage they serve (the Skill tool):

- **brainstorming**: before a new screen, flow or feature: explore intent and options with Theo.
- **frontend-design** or **design-taste-frontend**: visual direction, typography, avoiding
  templated defaults, once the wireframe is approved.
- **web-design-guidelines** and **fixing-accessibility**: review the result (contrast, target
  sizes, focus order, labels, states) before calling it done.
- **improve-ui**: when the task is to review or refine an existing design.
- **create-design-md**: when the project has no written design language and Theo wants one.

Their rules about code apply to the code that will implement the design; their design judgement
applies here.

## Rule 4: wireframes first

1. A **wireframe page** ("01 Wireframes"): greyscale, one font, boxes and real-ish copy, no colour,
   no imagery, no effects. Structure, hierarchy, flow and states only. For a flow, one frame per
   step, linked with prototype interactions when it helps.
2. Show it (screenshot) and ask for approval or changes. Offer 2 or 3 layouts when the structure is
   genuinely open, never 6.
3. Only once the wireframe is approved: the visual design, on its own page, built from components.

A request for "just the final design" still gets a quick wireframe question first: "A wireframe
before? It takes a few minutes and catches structure problems early." Theo may say no.

## Rule 5: build it like a design system

- **Auto layout everywhere.** Every frame that holds more than one layer uses auto layout
  (`layout`, `gap`, `padding`, `mainAlign`, `crossAlign`, `sizingH`/`sizingV`), nested as deep as
  the structure is. Absolute positioning only for overlays that truly float (a badge on an avatar,
  a tooltip), with `absolute: true` and constraints. No hand-placed x/y lists of children.
- **Components for anything used twice**, and for every UI element that has states (buttons,
  inputs, list rows, cards, tabs, navigation items). Variants for states and sizes (default, hover,
  pressed, focused, disabled, error, loading; small/medium/large) and component properties for text,
  icons and booleans. Screens use instances, never detached copies.
- **Variables and styles, no raw values.** Colours, spacing, radii and type come from variables
  (with modes for light/dark when needed) and text/effect styles. A hex code or a magic number on a
  layer in the final design is a bug: create the token first, then bind it.
- **A spacing scale** (4 or 8 based, or the project's) and a type scale, both stated in the file.
- **Naming**: layers, components, variants and pages named by role ("Button / Primary / Medium",
  "Order row", "Header"), never "Frame 12".
- **Responsive**: `fill` and `hug` sizing, min/max widths, and frames at each agreed breakpoint.
- **States**: empty, loading, error, long content and success for every screen that can have them.
- **Accessibility**: contrast at least 4.5:1 for text (3:1 large text and UI parts), targets at
  least 24×24 (44×44 on touch), focus states, never colour alone to carry meaning, labels on inputs.

Before creating anything, `list_commands` with a filter to find the right command, and
`inspect_document` to learn the file's existing components, styles and variables: reuse them.
Group related edits in `batch` (one undo step, with a clear label).

## Rule 6: show, verify, write it down

- After each meaningful step, `inspect_document` / `get_node` to check sizes, layout, bindings and
  instances (no raw colours, no detached instances, no layers outside auto layout), then
  `screenshot` and look at it yourself before showing Theo.
- Present: the screenshot, the decisions made and their sources, the open questions. Ask, then wait.
- Keep a **"Notes" page** (or a frame on the first page) with: the brief, Theo's answers, the
  decisions and their reasons, the sources, open questions. The next session starts from it.
- Save the file (`save_file`) at each approved milestone.

## Rule 7: hand off to the developers

Developer sessions build only what you mark ready, and read the repository even when FigCraft is
closed. At each milestone Theo approves:

1. **Ready for dev**: set `devStatus: readyForDev` on each approved top-level frame or section
   (`run_command` `node.set`; `inspect.readyForDev` lists them). Only after Theo's yes. Then file the
   build: `claude-todo add 'Build <frame name>' --notes '<file> · frame <id> · ready <date> · <what
   it covers, states included>'`. Changing a frame already marked: say so, and file
   `claude-todo add 'Update <frame name>' --notes '<what changed>'` (developers see the diff with
   `dev.compare`).
2. **Tokens**: export the variables into the repository, both forms, at their usual paths:
   `variable.export {format: "json", path: "<repository root>/design/tokens.json"}` and
   `variable.export {format: "css", path: "<repository root>/design/tokens.css"}` (absolute paths,
   inside the folder FigCraft was opened for). The code takes its tokens from these
   files; never let them lag behind the FigCraft file.
3. **DESIGN.md** at the repository root (load the `create-design-md` skill for its structure): the
   brief, decisions and why, the tokens (pointing to `design/tokens.*`), the components and their
   variants, the frames ready for dev with their ids, and the open questions. Same content as the
   Notes page; DESIGN.md is the copy developer sessions can always read.
4. Say which files changed (`design/`, `DESIGN.md`, the `.figcraft`) so Theo can commit them together.

## Checklist before saying "done"

- [ ] Every question that had more than one reasonable answer was asked, and answered by Theo.
- [ ] The wireframe was approved before the visual design.
- [ ] Every container uses auto layout; every repeated or stateful element is a component instance.
- [ ] No raw colour, spacing or font value outside variables and styles.
- [ ] Empty, loading, error and long-content states exist where they can happen.
- [ ] Contrast, target sizes and focus states checked (web-design-guidelines, fixing-accessibility).
- [ ] Best-practice sources cited for the main patterns.
- [ ] Notes page up to date; file saved.
- [ ] Approved frames marked ready for dev, each with its "Build" to-do.
- [ ] `design/tokens.json`, `design/tokens.css` and `DESIGN.md` updated.
- [ ] The `✎ design` to-dos you answered marked done (`claude-todo done <id>`).
