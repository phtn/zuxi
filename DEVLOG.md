# Zuxi Development and GPUI Learning Log

Last updated: 2026-07-30

This is a living record of how Zuxi works, why important decisions were made,
and what each change teaches about GPUI. It is intentionally more explanatory
than a release changelog.

The history below was reconstructed from the current working tree and the
development session. Once the repository has regular commits, link each future
entry to its commit or pull request.

## How to use this document

When reviewing a change:

1. Read its log entry to understand the outcome and reason.
2. Open the linked source files and follow the described data flow.
3. Run the app and change one small value to test your understanding.
4. Add a new log entry whenever behavior, architecture, dependencies, or a
   debugging assumption changes.

[PATTERNS.md](PATTERNS.md) is the short rulebook for state and component
choices. [NOTES.md](NOTES.md) records dependency constraints. This file
explains how those rules fit together in the working application.

Because this is a Markdown file inside the library folder, you can also run
Zuxi and select this log in its own sidebar.

## Zuxi in one paragraph

Zuxi is a two-crate Rust workspace. The `app` crate discovers Markdown and PDF
files, owns the selected-document state, and renders the window. The
`design_system` crate owns reusable tokens, semantic themes, and stateless UI
primitives. GPUI supplies the application lifecycle, entities, event callbacks,
layout, and painting. Markdown is parsed in process; PDF pages are converted to
PNG images by Poppler and then displayed by GPUI.

## Architecture map

- [Cargo.toml](Cargo.toml): workspace membership and pinned GPUI dependencies.
  Read it when build or platform behavior changes.
- [main.rs](crates/app/src/main.rs): startup, `RootView` state, events, layout,
  and preview rendering. Start here when tracing visible behavior.
- [document.rs](crates/app/src/document.rs): file discovery, loading, PDF
  caching, and Poppler commands. Read it when adding a file type or changing
  PDF behavior.
- [markdown.rs](crates/app/src/markdown.rs): converts parser events into Zuxi's
  simple block model. Read it when adding Markdown syntax.
- [tokens.rs](crates/design_system/src/tokens.rs): raw color, spacing, type,
  radius, and opacity values. Read it when changing visual constants.
- [theme.rs](crates/design_system/src/theme.rs): maps raw tokens to semantic
  light and dark roles. Read it when changing theme meaning.
- [components](crates/design_system/src/components): reusable `RenderOnce`
  primitives. Read these files when building shared UI.
- [showcase.rs](crates/design_system/examples/showcase.rs): a small executable
  for inspecting primitives. Use it to test the design system in isolation.
- [PATTERNS.md](PATTERNS.md): rules for `Entity` versus `RenderOnce`. Read it
  before introducing state.
- [NOTES.md](NOTES.md): GPUI pin and toolchain policy. Read it before upgrading
  dependencies.

## End-to-end data flow

```text
CLI folder argument
        |
        v
document::discover(root)
        |
        v
Vec<DocumentEntry> ------> sidebar rows
        |
        | selected index
        v
document::load(entry, cache)
        |
        +--> Markdown --> markdown::parse --> Vec<MarkdownBlock>
        |
        +--> PDF --> pdftoppm/pdfinfo --> cached PNG pages
        |
        v
DocumentPreview
        |
        +--> Empty
        +--> Markdown
        +--> Pdf
        +--> Error
        |
        v
RootView::render_preview
        |
        v
GPUI element tree --> layout --> paint
```

The theme has a separate reactive path:

```text
Theme button
    |
    v
Entity<Theme>::update
    |
    v
Theme::toggle + cx.notify()
    |
    v
RootView's theme observer
    |
    v
RootView renders again with new semantic colors
```

## The GPUI mental model used here

### 1. `App` owns application-wide facilities

`main` starts with `gpui_platform::application().run(...)`. The closure receives
`&mut App`, which is used to:

- create entities with `cx.new(...)`;
- register the active theme as a typed global;
- open the window;
- activate the application.

The app-global theme stores an `Entity<Theme>`, not a raw `Theme`. That gives
callers a handle to reactive state rather than a copied snapshot.

### 2. An `Entity<T>` is mutable state with identity

`RootView` and `Theme` are entities. An `Entity<T>` is a handle; the actual
value is managed by GPUI.

Use:

- `entity.read(cx)` for an immutable snapshot;
- `entity.update(cx, |value, cx| { ... })` for mutation;
- `cx.notify()` after a visible state change.

`RootView::new` observes the theme:

```rust
cx.observe(&theme, |_, _, cx| cx.notify()).detach();
```

When the theme notifies, the observer notifies `RootView`, causing the window
content to render again. `detach()` keeps the observation alive instead of
dropping the returned subscription immediately.

### 3. `Context<T>` connects callbacks to their owning entity

Methods such as `select_document` receive `&mut Context<Self>`. Calling
`cx.notify()` tells GPUI that this entity's rendered output is stale.

`cx.listener(...)` adapts an event closure so it receives `&mut RootView`
directly:

```rust
.on_click(cx.listener(move |this, _, _, cx| {
    this.select_document(index, cx);
}))
```

This is why the sidebar click handler can mutate `RootView` without manually
finding or locking it.

### 4. `Render` rebuilds a description of the UI

`impl Render for RootView` does not imperatively draw pixels. It returns nested
elements describing the current UI. After a notification, GPUI calls `render`
again, lays out the returned tree, and paints it.

Most nodes begin with `div()` and gain behavior through chained style methods:

```rust
div()
    .flex()
    .flex_col()
    .gap(px(tokens::spacing::XS))
    .bg(colors.surface)
    .child(...)
```

Think of this as a Rust builder API for a declarative element tree.

### 5. `RenderOnce` is for prop-driven components

The design-system `Button`, `Text`, `Icon`, `Container`, and `Stack` types use
`RenderOnce`. They receive all required data as fields and are consumed when
rendered. They do not own an independent mutable lifecycle.

This is a useful boundary:

- choose `Entity<T> + Render` when state must survive and notify observers;
- choose `RenderOnce` when values and callbacks can arrive as props.

### 6. `IntoElement`, `AnyElement`, and `SharedString` solve element typing

GPUI element builders have concrete Rust types.

- `IntoElement` lets a value become a renderable element.
- `AnyElement` type-erases different concrete element types. Zuxi uses it for
  helper methods and vectors whose branches return different element builders.
- `SharedString` is GPUI's inexpensive, cloneable string representation. It is
  useful when text crosses component boundaries or is captured by an element.

Prefer concrete return types when practical. Use `AnyElement` where branching
or heterogeneous collections make a single concrete type awkward.

### 7. Layout is flexbox-like, and scroll containers need constraints

The root is a horizontal flex container: a fixed-width sidebar plus a flexible
content column. The content column contains a fixed toolbar and a flexible
preview.

The repeated combination below is important:

```rust
.flex_1()
.min_h_0()
.overflow_y_scroll()
```

A flex child may otherwise keep its intrinsic minimum height and expand instead
of becoming scrollable. `min_h_0()` gives it permission to shrink into the
available space.

### 8. Platform features can decide whether painting exists at all

On macOS, `gpui_platform` must enable its `font-kit` feature. Without it, this
pinned GPUI revision installs a no-op text system. The app can still lay out
and paint backgrounds, borders, and images, which makes the failure look like a
Markdown or color bug even though the missing layer is platform text rendering.

The workspace dependency deliberately contains:

```toml
gpui_platform = { ..., features = ["font-kit"] }
```

Treat platform feature flags as part of the executable's behavior, not merely
as build configuration.

## Current implementation in more detail

### `RootView`: the application state holder

`RootView` currently owns:

- `theme: Entity<Theme>` is the reactive handle to semantic colors.
- `root: PathBuf` is the canonical folder being browsed.
- `documents: Vec<DocumentEntry>` is the current sorted library.
- `selected: Option<usize>` identifies the selected row when the library is
  non-empty.
- `preview: DocumentPreview` contains already-loaded content for the selected
  row.
- `_cache: TempDir` keeps rendered PDF PNGs alive for the view's lifetime.

The leading underscore on `_cache` communicates that the field is intentionally
kept for its lifetime side effect even though methods rarely read it directly.
Dropping `RootView` drops `TempDir`, which cleans up the PDF cache.

The current model is deliberately small and synchronous. If file watching,
background loading, tabs, or editing are added, `RootView` will be the first
place to reconsider state boundaries.

### Startup sequence

1. `library_root()` reads the optional first CLI argument.
2. The path is canonicalized so discovery and cache keys use a stable root.
3. `application().run(...)` creates the GPUI application.
4. `ActiveTheme::init(ThemeMode::Dark, cx)` creates and registers the theme.
5. A centered window is opened.
6. The window closure creates `RootView`.
7. `RootView::new` discovers documents and loads the first preview.

### Selection and reload

Selecting a sidebar row:

1. checks whether that row is already selected;
2. updates the selected index;
3. loads the matching preview;
4. calls `cx.notify()`.

Reloading is slightly more careful. It remembers the selected file's path,
discovers the folder again, and finds that path in the new list. This preserves
selection when sorting or neighboring files change. If the path disappeared,
the first available document becomes selected.

### Markdown pipeline

`pulldown-cmark` emits a stream of parser events. Zuxi reduces those events into
`MarkdownBlock` values:

- heading;
- paragraph;
- code block;
- quote;
- list item;
- horizontal rule.

`render_markdown_block` maps each block kind to a styled GPUI element.

This is a deliberately simplified document model, not a full Markdown AST.
Inline emphasis, links, nested lists, and tables do not preserve all of their
structure or styling. That simplicity makes the first renderer easy to learn,
but it identifies a future boundary: richer Markdown should introduce inline
runs or a tree rather than continually adding special cases to one flat string.

This log deliberately uses lists instead of Markdown tables so it remains
readable in Zuxi's current preview.

### PDF pipeline

PDF rendering is intentionally outside GPUI:

1. A cache key is derived from path, file size, and modification time.
2. `pdftoppm` rasterizes up to 24 pages at 120 DPI.
3. `imagesize` reads each PNG's dimensions for its aspect ratio.
4. `pdfinfo` reports the source document's total page count.
5. GPUI displays each PNG with `img(...).object_fit(ObjectFit::Contain)`.

The cache avoids rerasterizing a PDF during the current app session. Including
file metadata in the key prevents a changed PDF from reusing stale pages.

The subprocess calls are synchronous today. This is acceptable for the current
small viewer, but large files could block the UI thread. Moving document work
to a background task is a natural future GPUI learning exercise.

### Design-system layering

The visual system has two levels:

1. Raw tokens answer "what exact value is this?"
2. Semantic theme colors answer "what job does this value perform?"

For example, application code requests `colors.surface_elevated`; it does not
request a literal gray. This lets light and dark themes assign different raw
values without rewriting components.

The current dark hierarchy is:

- background: `#232426`;
- surface: `#2c2c2e`;
- elevated surface: `#3a3a3c`;
- border: `#48484a`.

Large fills remain neutral, like Finder. System blue is reserved for semantic
accents such as focus and quote markers.

## Change log

Entries are newest first.

### 2026-07-30 — Fixed the `cargo dev` watch alias

**Outcome:** `cargo dev` now starts `cargo-watch` with `run -p app` as one
complete command argument.

**Files:**

- `.cargo/config.toml`

**Observed symptom:**

```text
unexpected EOF while looking for matching `''
syntax error: unexpected end of file
```

**Root cause:** The string-form Cargo alias contained shell-style single quotes.
Cargo's alias expansion passed those quotes through in a form that caused
`cargo-watch` to invoke the malformed command `cargo 'run`.

**Fix:** Use Cargo's argument-array form:

```toml
dev = ["watch", "-x", "run -p app"]
```

**GPUI lesson:** This was outside GPUI itself. When the application never
reaches compilation or startup, debug the command runner before investigating
window or entity lifecycle code.

**Verification:** Started `cargo dev`, confirmed that `cargo-watch` invoked
`cargo run -p app`, and stopped the watcher cleanly.

### 2026-07-30 — Added this development and learning log

**Outcome:** Added a durable place to review project changes and learn the GPUI
concepts exercised by Zuxi.

**Files:**

- `DEVLOG.md`
- `README.md`

**Reasoning:** A terse changelog would record outcomes but not build a mental
model. This file combines history, architecture, debugging lessons, limitations,
and a repeatable entry format.

**GPUI lesson:** Documentation is most useful when it describes the reactive
path from state mutation to rendering, not just the final pixels.

### 2026-07-29 — Reworked both themes around Finder-style window colors

**Outcome:** Removed slate-blue panel fills and introduced a neutral macOS-like
surface hierarchy. Sidebar selection, hover states, toolbar badges, buttons,
and code blocks now use neutral grays. Blue is limited to semantic accents.
Rounded treatments were added to interactive rows, badges, buttons, and code
blocks.

**Files:**

- `crates/design_system/src/tokens.rs`
- `crates/design_system/src/theme.rs`
- `crates/design_system/src/components/button.rs`
- `crates/app/src/main.rs`

**Reasoning:** Changing only the main background left the sidebar, toolbar, and
elevated content in the old slate palette. Theme cohesion requires updating all
semantic surface roles and avoiding accent colors for large selection fills.

**GPUI lesson:** Theme roles are the right abstraction boundary. Once components
consume semantic colors, a broad visual correction can remain centralized.

**Verification:**

- visually compared the dark appearance with Finder;
- visually checked Markdown in light and dark modes;
- added a test that dark surfaces stay neutral and correctly layered;
- ran formatting, workspace checks, Clippy, and tests.

### 2026-07-29 — Fixed missing letters in Markdown and the rest of the UI

**Outcome:** Enabled `gpui_platform`'s macOS `font-kit` feature. Text now renders
in the toolbar, sidebar, Markdown preview, and design-system components.

**Files:**

- `Cargo.toml`

**Observed symptom:** Layout boxes, borders, code-block fills, and PDF images
appeared, but glyphs did not. The Markdown parser was producing valid blocks.

**Root cause:** `gpui_platform` has no default `font-kit` feature. In this GPUI
revision, macOS creates a `NoopTextSystem` when that feature is absent.

**GPUI lesson:** Debug rendering in layers:

1. verify data exists;
2. verify layout has space;
3. verify colors and clipping;
4. verify the platform renderer and required features.

When all text is absent but non-text primitives paint, investigate the text
system before rewriting parsing or layout code.

**Verification:** Rebuilt the app and visually confirmed Markdown headings,
paragraphs, list content, and code text.

### 2026-07-29 — Established the initial document-viewer vertical slice

**Outcome:** Created a working Rust workspace with:

- a reusable design-system crate;
- a GPUI application window;
- folder discovery and sidebar selection;
- simplified Markdown parsing and rendering;
- cached PDF raster previews;
- light and dark themes;
- a design-system showcase;
- CI and local validation commands.

**Key architectural decision:** Keep reusable visual foundations in
`design_system`, while document-domain state and rendering remain in `app`.

**GPUI lesson:** A vertical slice is more informative than a large speculative
component library. The document viewer exercises application state, events,
theme observation, layout, text, images, scrolling, and external-process
integration together.

## Known limitations and useful future exercises

### Document loading blocks the UI thread

Discovery, Markdown reads, and Poppler subprocesses run synchronously. Learn
GPUI tasks by moving expensive preview loading off the UI thread and returning
the result to the `RootView` entity.

### Markdown rendering is block-level only

The parser flattens inline structure. A good next exercise is preserving inline
code, emphasis, and links as styled text runs while keeping block layout
separate.

### The library updates only when Reload is clicked

A filesystem watcher would teach event delivery from an external source. Keep
the watcher state outside `RenderOnce` components and notify `RootView` when
the discovered library changes.

### `RootView` owns every application concern

This is appropriate at the current size. If preview loading gains asynchronous
states, consider a dedicated preview entity with states such as idle, loading,
ready, and failed. Split state when another lifecycle genuinely exists, not
merely to shorten a file.

### PDF support depends on installed command-line tools

The app expects `pdftoppm` and `pdfinfo`. The error path explains how to install
Poppler, but there is no in-app capability check at startup.

### Accessibility is minimal

Interactive elements have stable IDs, but the application has not yet been
audited for roles, labels, keyboard navigation, or focus behavior. This would
be a valuable GPUI exercise after the basic document experience is stable.

## Debugging playbook

Use the visible symptom to choose the first layer to inspect:

- File absent from sidebar: check the extension filter, hidden-directory
  filter, and discovery root.
- Wrong Markdown title: check first-H1 extraction and the fallback file stem.
- Sidebar click does nothing: check the element ID, `cx.listener`, selected
  index, and `cx.notify()`.
- State changes but UI stays stale: check for a missing observer or
  notification.
- Preview has no height: check flex constraints, `flex_1`, and `min_h_0`.
- Scroll does not engage: check the constrained parent height and
  `overflow_y_scroll`.
- All text is missing: check `gpui_platform/font-kit`, font resolution, and
  clipping.
- Only one Markdown construct is wrong: check parser events and the
  `MarkdownBlockKind` render mapping.
- PDF is stale: check cache-key metadata and temporary cache contents.
- PDF fails entirely: check Poppler availability and subprocess stderr.
- Theme looks inconsistent: look for a hardcoded color or the wrong semantic
  theme role.

## Suggested reading path for a new GPUI developer

### Pass 1: Follow startup

Read the bottom of `main.rs`, then `RootView::new`. Be able to answer:

- Where is the theme entity created?
- Where is the window created?
- When are documents first loaded?

### Pass 2: Follow one click

Start at the sidebar's `.on_click(...)` and follow:

```text
listener -> select_document -> document::load -> cx.notify -> render
```

Change the selected-row radius and confirm that only presentation changed.

### Pass 3: Follow the theme

Start at the Theme button and follow:

```text
Entity::update -> Theme::toggle -> Theme::set_mode -> notify -> observer
```

Change one semantic token and use the showcase to see which components consume
that role.

### Pass 4: Follow Markdown

Use a small file containing one heading, paragraph, quote, list, rule, and code
block. Step through `markdown::parse`, then find each branch in
`render_markdown_block`.

### Pass 5: Follow a PDF

Trace `document::load` through `render_pdf`, the cache key, `pdftoppm`, aspect
ratio calculation, and finally the `img` element.

## Questions to test your mental model

1. Why is `Theme` an entity while `Button` is a `RenderOnce` component?
2. Why does toggling the theme re-render `RootView`?
3. What visible bug could occur if `cx.notify()` were removed from
   `select_document`?
4. Why is `_cache: TempDir` stored on `RootView`?
5. Why does the preview column use `min_h_0()`?
6. Which layer would you change to add a new raw gray?
7. Which layer would you change to make every elevated surface darker?
8. Why can `AnyElement` be useful in `render_preview`?
9. What information is lost by the current flat Markdown block model?
10. Why did a missing Cargo feature look like a Markdown rendering bug?

If you can answer these from the code, you have the core mental model for the
current application.

## Validation commands

Run the same checks locally and in CI:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For a visual check:

```sh
cargo run -p app
cargo run -p design_system --example showcase
```

## Template for future entries

Copy this section near the top of the change log:

```markdown
### YYYY-MM-DD — Short outcome

**Outcome:** What is visibly or behaviorally different?

**Files:**

- `path/to/file.rs`

**Reasoning:** Why was this approach chosen? What alternative was rejected?

**GPUI lesson:** Which lifecycle, state, event, layout, or rendering concept did
this exercise?

**Verification:** Which automated checks and visual scenarios passed?

**Follow-up:** What limitation or next question remains?
```
