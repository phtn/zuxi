# Zuxi State and Component Patterns

Use a GPUI `Entity` for state that has identity, changes over time, or must notify
multiple readers. Keep display-only values in `RenderOnce` component props.

## Shared state: Entity

An entity owns mutable state. Update it through its handle and call `notify` so
observers re-render:

```rust
use gpui::{App, AppContext, Context, Entity};

struct Counter {
    value: usize,
}

fn make_counter(cx: &mut App) -> Entity<Counter> {
    cx.new(|_| Counter { value: 0 })
}

fn increment(counter: &Entity<Counter>, cx: &mut App) {
    counter.update(cx, |counter, cx| {
        counter.value += 1;
        cx.notify();
    });
}
```

A view that renders entity state should observe the entity during construction:

```rust
cx.observe(&counter, |_, _, cx| cx.notify()).detach();
```

The active design-system theme follows this pattern and is also registered as a
typed global through `ActiveTheme`, allowing any root view to acquire its handle.

## Stateless UI: RenderOnce

Use a `RenderOnce` component when all input arrives as props and the component does
not need independent lifecycle or mutable state:

```rust
use gpui::{IntoElement, RenderOnce, SharedString, Window, div, prelude::*};

#[derive(IntoElement)]
struct Label {
    value: SharedString,
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().child(self.value)
    }
}
```

Before adding an entity, ask whether another part of the app must observe the value.
If not, prefer a prop or a closure on a `RenderOnce` component.
