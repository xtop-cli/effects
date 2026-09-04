# Writing an effect for xtop

How to implement a new effect against the shared contract crate
`xtop-effect-api`, which lives in the `api` repository and is the single
source of truth for the effect contract (DR-5 in the ecosystem ROADMAP).
The reference implementation in this repository is `xtop-effect-fade`.

## The contract

Two public items, and deliberately nothing else:

```rust
// xtop-effect-api
pub struct EffectManifest {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub trait Effect: Send + Sync {
    fn manifest(&self) -> EffectManifest;
    fn on_frame(&mut self, buffer: &mut ratatui::buffer::Buffer,
                elapsed: std::time::Duration);
}
```

There is no registration or glue type: an effect is just a `Send + Sync`
type implementing `Effect`. The kernel holds concrete effects behind an
optional feature and calls the trait.

## Host invocation contract

The kernel (`xtop`) is the host. Its obligations, as documented in the
`xtop-effect-api` crate docs, are:

- `Effect::on_frame` is called on **every rendered frame**, after widgets
  and layout have been drawn into the ratatui buffer and **before** the
  terminal flush.
- `elapsed` is the time since the effect started — an absolute age, **not**
  a frame delta.
- Effects are **stateful across frames**: any progress an effect needs is
  kept on `self` between calls.
- The host does **not** pace frames (it ticks at roughly one-second cadence
  plus ad-hoc repaints). A transition must therefore advance by wall-clock
  time (`elapsed`), never by counting frames.

An effect never depends on the kernel; it uses only `xtop-effect-api` and
ratatui's buffer type.

## What the effect side must guarantee

1. `manifest()` returns stable, static metadata: a short `id` (used to
   select the effect), a human `name`, and a one-line `description`.
2. `on_frame` may rewrite any cells of the fully rendered frame in place.
3. After an effect's window has elapsed it must become a no-op: later
   frames stay byte-identical to the host-rendered buffer.
4. The same `(buffer, elapsed)` pair must produce the same output.
   Determinism is what makes effects testable without a time source.

## The pure-function pattern

Keep progress out of the effect when the semantics allow it. `elapsed` is
absolute, so an effect can be a pure function of `(buffer, elapsed)` — the
implementation below is exactly what `xtop-effect-fade` does:

```rust
use std::time::Duration;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use xtop_effect_api::{Effect, EffectManifest};

pub const WINDOW: Duration = Duration::from_millis(500);

pub struct MyEffect;

impl Effect for MyEffect {
    fn manifest(&self) -> EffectManifest {
        EffectManifest {
            id: "my-effect",
            name: "My effect",
            description: "What it does, and over what window",
        }
    }

    fn on_frame(&mut self, buffer: &mut Buffer, elapsed: Duration) {
        if elapsed >= WINDOW {
            return; // done: leave the frame byte-identical
        }
        // derive everything from `elapsed`; no timers, no frame counters
        for cell in &mut buffer.content {
            // rewrite only what must change (e.g. RGB colors, per the
            // reference effect); keep Color::Reset and modifiers intact
            let _ = &cell.fg; // Cell fields fg/bg/modifier/symbol are public
        }
    }
}
```

Only reach for `self` state when the visual genuinely needs history that
`elapsed` cannot express. Stateful or not, the type must stay `Send + Sync`.

## Testing pattern

Because the effect is a pure function of `(buffer, elapsed)`, tests need no
terminal, no time source and no host:

- Build a buffer with `ratatui::buffer::Buffer::empty(Rect::new(0, 0, w, h))`
  and paint a few cells of interest through the public `content` field.
- Call `effect.on_frame(&mut buffer, some_duration)`.
- Assert on `buffer.content()` — cell fields (`fg`, `bg`, `modifier`,
  `symbol`) all implement `PartialEq`, so whole buffers compare with
  `assert_eq!(a.content(), b.content())`.

The `xtop-effect-fade` test suite covers the boundary cases every effect
should test: `elapsed == 0` (start state), a mid-window value, exactly the
window, repeated calls past the window (stability), and an empty buffer.
`Buffer: Clone` makes the byte-identical checks one line:

```rust
let before = buffer.clone();
effect.on_frame(&mut buffer, WINDOW);
assert_eq!(before.content(), buffer.content());
```

## Landing a new effect in this repository

1. Create `xtop-effect-<name>/` with a manifest inheriting the workspace
   fields (`version.workspace`, `edition.workspace`, ...), depending on
   `ratatui` and `xtop-effect-api` through `[workspace.dependencies]`.
2. Add the crate to `members` in the root `Cargo.toml`.
3. Implement `Effect`, mirror the testing pattern above, and document the
   exact visual semantics in the crate docs.
4. Add a catalog entry in `docs/effects.md` and a row in the README table.

Shared helper code belongs in an `effects-lib/` crate — but only once a
second effect actually shares code with an existing one. Do not create it
empty.
