//! `xtop-effect-fade` — the Fade effect for xtop.
//!
//! Fade implements the effect contract from `xtop-effect-api`: it receives
//! the fully rendered ratatui buffer of one frame plus the time since the
//! effect started, and blends the frame in from black over a fixed window.
//!
//! # Visual semantics
//!
//! - The fade window is [`FADE_DURATION`]: 500 ms.
//! - Every `Color::Rgb` foreground and background in the buffer is scaled
//!   toward black: a channel `c` becomes `round(c * alpha)` with
//!   `alpha = elapsed / FADE_DURATION` clamped to `[0, 1]`.
//! - At `elapsed == 0` the whole RGB content is black; at
//!   `elapsed >= FADE_DURATION` the frame is at full intensity.
//! - Colors that are not `Color::Rgb` (`Color::Reset`, indexed colors,
//!   grayscale) and all cell modifiers are left untouched: the effect only
//!   ever rewrites the `fg`/`bg` of cells that carry an RGB color.
//!
//! Result, stated honestly: RGB content fades in from black over half a
//! second. On a terminal with a dark default background the frame appears
//! from black; on a light default background the `Color::Reset` backdrop is
//! visible immediately while RGB colors still ramp up from black.
//!
//! # Determinism
//!
//! Fade keeps no progress state: [`on_frame`](Effect::on_frame) is a pure
//! function of the buffer and `elapsed`. The same inputs always produce the
//! same output; no time source or randomness lives inside the effect.
//!
//! - `elapsed >= FADE_DURATION`: [`on_frame`](Effect::on_frame) returns
//!   without touching the buffer (zero writes), so every later frame is
//!   byte-identical to the host-rendered frame.
//! - `elapsed < FADE_DURATION`: deterministic per-channel scaling as above.
//!
//! The effect has no configuration: the window is the constant
//! [`FADE_DURATION`] and the host drives it purely through `elapsed` (see
//! the `xtop-effect-api` crate docs for the full host contract).

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::style::Color;
use xtop_effect_api::{Effect, EffectManifest};

/// Length of the fade-in window, measured from the moment the effect starts.
pub const FADE_DURATION: Duration = Duration::from_millis(500);

/// Static metadata reported by [`FadeEffect`] through
/// [`Effect::manifest`].
const MANIFEST: EffectManifest = EffectManifest {
    id: "fade",
    name: "Fade",
    description: "Fades the rendered frame in from black over 500 ms",
};

/// Fade — fades a rendered frame in from black over 500 ms.
///
/// Implements [`Effect`] from `xtop-effect-api`. It is a unit struct with no
/// configuration and no internal progress state: the fade advance is fully
/// derived from the `elapsed` value the host passes in, which makes the
/// effect deterministic and trivially testable (see the [crate docs](crate)
/// for the exact scaling semantics and guarantees).
#[derive(Debug, Clone, Copy, Default)]
pub struct FadeEffect;

impl Effect for FadeEffect {
    fn manifest(&self) -> EffectManifest {
        MANIFEST
    }

    fn on_frame(&mut self, buffer: &mut Buffer, elapsed: Duration) {
        // Past the window the effect is done: leave the frame byte-identical.
        if elapsed >= FADE_DURATION {
            return;
        }
        let alpha = elapsed.as_secs_f32() / FADE_DURATION.as_secs_f32();
        for cell in &mut buffer.content {
            if let Color::Rgb(r, g, b) = cell.fg {
                cell.fg = Color::Rgb(
                    scale_channel(r, alpha),
                    scale_channel(g, alpha),
                    scale_channel(b, alpha),
                );
            }
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    scale_channel(r, alpha),
                    scale_channel(g, alpha),
                    scale_channel(b, alpha),
                );
            }
        }
    }
}

/// Linear interpolation of one 8-bit channel toward black by factor
/// `alpha`: `round(channel * alpha)`. Deterministic for fixed inputs.
fn scale_channel(channel: u8, alpha: f32) -> u8 {
    (f32::from(channel) * alpha).round() as u8
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use xtop_effect_api::Effect;

    use super::{FadeEffect, FADE_DURATION};

    /// A small buffer with a mixed set of colors:
    /// cell 0: `Color::Reset` foreground (untouched by the effect),
    /// cell 1: RGB foreground + RGB background (dimmed by the effect),
    /// cell 2: indexed foreground (untouched by the effect).
    fn sample_buffer() -> Buffer {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 3, 1));
        buffer.content[0].fg = Color::Reset;
        buffer.content[1].fg = Color::Rgb(200, 150, 100);
        buffer.content[1].bg = Color::Rgb(10, 20, 30);
        buffer.content[2].fg = Color::Indexed(5);
        buffer.content[2].bg = Color::Reset;
        buffer
    }

    #[test]
    fn completed_fade_leaves_the_buffer_byte_identical() {
        let mut effect = FadeEffect;
        let before = sample_buffer();
        let mut after = before.clone();

        effect.on_frame(&mut after, FADE_DURATION);

        assert_eq!(before.area(), after.area());
        assert_eq!(before.content(), after.content());
    }

    #[test]
    fn zero_elapsed_dims_rgb_cells_and_leaves_other_colors_alone() {
        let mut effect = FadeEffect;
        let original = sample_buffer();
        let mut faded = original.clone();

        effect.on_frame(&mut faded, Duration::ZERO);

        // RGB colors are dimmed to black at the very start of the fade.
        assert_eq!(faded.content()[1].fg, Color::Rgb(0, 0, 0));
        assert_eq!(faded.content()[1].bg, Color::Rgb(0, 0, 0));
        // Reset and indexed cells are untouched.
        assert_eq!(faded.content()[0].fg, Color::Reset);
        assert_eq!(faded.content()[2].fg, Color::Indexed(5));
        assert_eq!(faded.content()[2].bg, Color::Reset);
        // The buffer as a whole differs from the original.
        assert_ne!(faded.content(), original.content());
    }

    #[test]
    fn mid_fade_state_is_strictly_between_start_and_end() {
        let mut start_effect = FadeEffect;
        let full = sample_buffer();
        let mut at_zero = full.clone();
        start_effect.on_frame(&mut at_zero, Duration::ZERO);

        let mut mid_effect = FadeEffect;
        let mut at_mid = full.clone();
        mid_effect.on_frame(&mut at_mid, FADE_DURATION / 2);

        // alpha == 0.5 exactly: 200 -> 100, 150 -> 75, 100 -> 50 (and
        // 10 -> 5, 20 -> 10, 30 -> 15), so every channel is strictly
        // between its start (black) and end (full) value.
        assert_eq!(at_zero.content()[1].fg, Color::Rgb(0, 0, 0));
        assert_eq!(at_mid.content()[1].fg, Color::Rgb(100, 75, 50));
        assert_eq!(at_mid.content()[1].bg, Color::Rgb(5, 10, 15));
        assert_eq!(full.content()[1].fg, Color::Rgb(200, 150, 100));
        // And the mid-fade buffer differs from both endpoints as a whole.
        assert_ne!(at_mid.content(), at_zero.content());
        assert_ne!(at_mid.content(), full.content());
    }

    #[test]
    fn manifest_is_stable_across_instances() {
        let manifest = FadeEffect.manifest();
        assert_eq!(manifest.id, "fade");
        assert_eq!(manifest.name, "Fade");
        assert_eq!(
            manifest.description,
            "Fades the rendered frame in from black over 500 ms"
        );
        assert_eq!(manifest, FadeEffect.manifest());
    }

    #[test]
    fn repeated_calls_after_the_window_keep_the_buffer_stable() {
        let mut effect = FadeEffect;
        let expected = sample_buffer();
        let mut buffer = expected.clone();

        // Exactly at the window and twice past it: the frame must stay
        // byte-identical to the host-rendered buffer every time.
        effect.on_frame(&mut buffer, FADE_DURATION);
        effect.on_frame(&mut buffer, Duration::from_secs(1));
        effect.on_frame(&mut buffer, FADE_DURATION * 10);

        assert_eq!(buffer.area(), expected.area());
        assert_eq!(buffer.content(), expected.content());
    }

    #[test]
    fn empty_buffers_are_fine_at_any_elapsed() {
        let mut effect = FadeEffect;
        let mut buffer = Buffer::empty(Rect::new(0, 0, 0, 0));

        effect.on_frame(&mut buffer, Duration::ZERO);
        effect.on_frame(&mut buffer, FADE_DURATION / 2);
        effect.on_frame(&mut buffer, FADE_DURATION * 3);

        assert_eq!(buffer.content().len(), 0);
    }
}
