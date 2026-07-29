# Agent Plan: Initialize a Rust + GPUI Project with a Design System

This plan is written to be handed to a coding agent (e.g. Claude Code) and executed
end-to-end, with checkpoints where a human should review before continuing.

---

## Phase 0 — Preconditions & Environment Check

**Goal:** confirm the machine can actually build GPUI apps before writing code.

1. Confirm OS: GPUI mainline targets **macOS and Linux only**. Windows/other
   platforms require a community fork (`gpui-ce`) — decide now which you're on,
   since it changes the dependency line in Phase 1.
2. Confirm Rust toolchain:
   ```
   rustup update stable
   rustc --version   # should be recent stable, GPUI tracks stable closely
   ```
3. macOS only: confirm Xcode + command line tools are installed (Metal is the
   rendering backend).
   ```
   xcode-select --install
   ```
4. Linux only: confirm Vulkan/X11 or Wayland dev headers are present (varies by
   distro — check GPUI's README for the current list, this shifts between
   releases since it's pre-1.0).

**Checkpoint:** agent reports OS, Rust version, and backend confirmed before
proceeding. Do not continue silently if any check fails.

---

## Phase 1 — Scaffold the Project

**Goal:** a workspace that separates the *design system* from the *application*,
so the design system is reusable/testable independently.

1. Create a Cargo workspace, not a single crate:
   ```
   my-app/
   ├── Cargo.toml            # workspace root
   ├── crates/
   │   ├── design_system/    # tokens, theme, reusable components
   │   └── app/               # the actual application, depends on design_system
   ```
2. Root `Cargo.toml`:
   ```toml
   [workspace]
   members = ["crates/design_system", "crates/app"]
   resolver = "2"
   ```
3. Add GPUI as a dependency. Because GPUI is pre-1.0, pin to a git rev rather
   than a loose version to avoid silent breaking changes:
   ```toml
   [dependencies]
   gpui = { git = "https://github.com/zed-industries/zed", rev = "<pin-a-specific-commit>" }
   ```
   Agent should fetch the latest commit hash at plan-execution time and record
   it in a `NOTES.md` so upgrades are a deliberate, reviewed action later.
4. `cargo check` on an empty `fn main() {}` in `crates/app` to confirm the
   dependency resolves and compiles before writing any UI code. This isolates
   "environment problem" from "code problem" failures early.

**Checkpoint:** clean `cargo check` across the workspace.

---

## Phase 2 — Design System Foundation (`crates/design_system`)

**Goal:** tokens and theme types that both the design system's own components
and the app consume — single source of truth, no hardcoded colors/spacing in
app code.

1. **Tokens module** (`src/tokens.rs`): raw values only, no GPUI types leaking
   in unless needed for color type interop.
   - Color palette (base + semantic aliases: `background`, `surface`,
     `text_primary`, `text_muted`, `border`, `accent`, `danger`, etc.)
   - Spacing scale (e.g. 4/8/12/16/24/32/48px as named constants, not magic
     numbers)
   - Typography scale (font sizes, weights, line-heights)
   - Radii, shadows if applicable

2. **Theme module** (`src/theme.rs`): a `Theme` struct built from tokens,
   with support for at least light/dark variants from day one — retrofitting
   theming later is expensive. Store the active theme as a GPUI Entity so
   components can react to theme changes.

3. **Primitive components** (`src/components/`): the smallest reusable
   building blocks, each as a `RenderOnce` element:
   - `Text`, `Button`, `Icon`, `Container`/`Box`, `Stack` (row/column with
     spacing token support)
   - Each primitive takes theme tokens as input rather than hardcoding
     colors — verify this discipline in review, it's the easiest thing to
     drift on.

4. Write a minimal `cargo test` or example binary in `design_system` that
   renders a few primitives in isolation, so the design system can be
   validated without booting the full app.

**Checkpoint:** human review of the token set and theme structure before
components are built on top — this is the layer most expensive to change
later.

---

## Phase 3 — Application Bootstrap (`crates/app`)

**Goal:** the smallest possible running window, wired to the design system.

1. `main.rs`:
   ```rust
   use gpui::{Application, App, WindowOptions};

   fn main() {
       Application::new().run(|cx: &mut App| {
           cx.open_window(WindowOptions::default(), |window, cx| {
               // root view goes here
           });
       });
   }
   ```
2. Register a root view that renders a single `design_system` primitive
   (e.g. a themed `Text` element) — proves the two crates are wired
   correctly end to end before building real UI.
3. Wire the `Theme` entity into the app's context so descendant views can
   read it (this is the pattern to settle now, since retrofitting global
   theme access later touches every component).

**Checkpoint:** `cargo run` opens a window showing themed content. This is
the "hello world" milestone — don't proceed to real UI until this is solid.

---

## Phase 4 — State Management Pattern

**Goal:** decide and document the Entity/state pattern before the app grows,
so components aren't refactored later.

1. Identify what's genuinely shared app state (Entities) vs. local component
   state (closures/RenderOnce props).
2. Write one example of each in a `PATTERNS.md`:
   - A stateful entity (e.g. a counter or settings panel) showing
     read/update/notify flow.
   - A stateless `RenderOnce` component receiving props only.
3. This doc becomes the reference the agent (or future contributors) checks
   before adding new components, to avoid inconsistent state patterns.

**Checkpoint:** human review of `PATTERNS.md` — this is a cheap doc that
prevents expensive architectural drift.

---

## Phase 5 — Dev Loop & Tooling

1. Add `cargo watch` or equivalent for fast iteration:
   ```
   cargo install cargo-watch
   cargo watch -x 'run -p app'
   ```
2. Set up a basic CI check (even local pre-commit) running:
   - `cargo check --workspace`
   - `cargo clippy --workspace`
   - `cargo test --workspace`
3. Record the pinned GPUI git rev and update policy in `NOTES.md` from Phase 1
   — GPUI breaks between versions often, so upgrades should be their own
   reviewed commit, not incidental.

**Checkpoint:** green `cargo check`, `clippy`, `test` before calling init done.

---

## Phase 6 — First Real Feature (Vertical Slice)

Rather than building out the whole design system speculatively, pick one real
screen/feature and build it fully — this validates tokens, theme, primitives,
and state pattern together, and surfaces gaps in the design system while
they're still cheap to fix.

1. Agent proposes 2–3 small candidate features.
2. Human picks one.
3. Agent implements it using only what exists in `design_system` — any gap
   found (missing primitive, missing token) gets added to the design system
   properly, not hacked into the app crate.

**Checkpoint:** working vertical slice + a short retro noting any design
system gaps discovered, filed as follow-up tasks.

---

## Explicit non-goals for this init pass

- No cross-platform (Windows) support unless you're on `gpui-ce` — flag this
  decision, don't silently assume.
- No component library beyond primitives — don't build a full kit before a
  real feature has exercised the pattern.
- No premature abstraction of the theme system for multi-brand/white-label
  use unless that's an actual near-term requirement.

---

## Summary checklist for the executing agent

- [ ] Phase 0: environment verified
- [ ] Phase 1: workspace scaffolded, clean `cargo check`
- [ ] Phase 2: tokens + theme + primitives, human-reviewed
- [ ] Phase 3: window boots, renders themed content
- [ ] Phase 4: state pattern documented
- [ ] Phase 5: dev loop + CI checks green
- [ ] Phase 6: one real feature built end-to-end, gaps logged
