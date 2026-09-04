# Effect catalog

Built-in frame effects shipped by this repository. All of them implement
the `xtop-effect-api` contract (see `docs/write-effect.md` for the authoring
guide and the host contract). The catalog is additive: one crate per
effect, one row per effect.

## Fade (`xtop-effect-fade`, manifest id `fade`)

The fade-in effect. Type: `xtop_effect_fade::FadeEffect` — a stateless unit
struct with no configuration of any kind.

### What it looks like

Fade takes the fully rendered frame of one tick and blends its RGB content
in from black. During the window every `Color::Rgb` foreground and
background is scaled toward black; colors that are not RGB (`Color::Reset`,
indexed, grayscale) and all cell modifiers are never touched.

- `elapsed == 0`: all RGB channels are zero — the frame is black where the
  widgets painted RGB colors.
- `elapsed` strictly between 0 and the window: every RGB channel sits at
  `round(channel * alpha)` with `alpha = elapsed / 500 ms`.
- `elapsed >= 500 ms`: full intensity.

On a terminal with a dark default background this reads as the whole frame
fading in from black over half a second. On a light default background the
`Color::Reset` backdrop (for example the empty space between text) is
visible immediately, while RGB text and backgrounds still ramp up from
black.

### Exact window semantics

- Window: exactly `FADE_DURATION = 500 ms` (a `const` in the crate),
  measured from the moment the effect started, i.e. from the `elapsed`
  value the host passes on each frame.
- The effect keeps no progress state: it is a pure function of the frame
  buffer and `elapsed`, so it never drifts with frame pacing or tick
  jitter.
- Determinism guarantee: once `elapsed >= FADE_DURATION`,
  `Effect::on_frame` returns without writing a single cell. Every later
  frame is byte-identical to the host-rendered buffer, for as long as the
  effect stays active — repeated calls are stable, and the buffer can be
  compared cell-for-cell (the test suite proves it).

### Testing

The suite in `xtop-effect-fade` covers: end-of-window byte-identity,
zero-elapsed dimming (RGB cells dimmed, `Color::Reset`/indexed cells
untouched), strict monotonicity at the mid-fade point, stable manifest
metadata, repeated calls past the window, and empty buffers.

## Integration note

The kernel (`xtop` repo) wires built-in effects into its render pipeline
behind an optional `effects` feature (ecosystem ROADMAP M5.3, delivered
with the kernel milestone): the draw closure applies the active effect's
`on_frame` to the buffer after layout and before the terminal flush,
passing the time since the effect started. Until that wiring lands, this
repository compiles and tests standalone against the contract crate
`xtop-effect-api`; nothing here depends on the kernel.
