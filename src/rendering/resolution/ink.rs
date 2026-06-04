//! Per-channel ink-routing stage.
//!
//! Subsumes the role of `separation_renderer.rs:714-822` (`tint_for_ink`):
//! given a fully-resolved colour and a target ink, decide whether the backend
//! paints into the plate (and at what tint) or skips it.
//!
//! Today this stage is dead code at the integration layer — the separation
//! renderer still uses its own `tint_for_ink`. The stage is here so that when
//! the separation backend migrates onto the pipeline (follow-up branch) the
//! per-plate decision can be taken by reading the [`ResolvedColor`]
//! produced by [`super::ColorResolver`] plus the [`OverprintPlan`] produced
//! by [`super::OverprintResolver`] without re-walking the source colour
//! space.

use crate::content::graphics_state::GraphicsState;

use super::resolved::{InkName, OverprintPlan, ResolvedColor};

pub(crate) struct InkRouter;

/// Per-plate decision returned by [`InkRouter::route`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InkAction {
    /// Paint into the target plate with the given tint (0.0 = knock out the
    /// plate at the touched pixels; 1.0 = full ink coverage).
    Paint(f32),
    /// Leave the target plate completely untouched (overprint-skip).
    Skip,
}

impl InkRouter {
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Decide what to do with `target_ink` for the given resolved colour.
    ///
    /// Implements the decision tree from ISO 32000-1:2008 §11.7.4:
    ///
    /// - If the colour's participating channel set names `target_ink`, paint
    ///   with the channel value.
    /// - If it doesn't and overprint is enabled, leave the plate untouched.
    /// - If it doesn't and overprint is disabled (the spec default), paint
    ///   0.0 — "areas of unspecified colorants are erased" (the per-plate
    ///   knockout rule).
    /// - For OPM=1 sources, a zero-valued channel for `target_ink` means
    ///   "colorant not specified" — leave the plate untouched even when
    ///   the channel is in the participating set.
    pub(crate) fn route(
        &self,
        _gs: &GraphicsState,
        target_ink: &InkName,
        color: &ResolvedColor,
        overprint: &OverprintPlan,
    ) -> InkAction {
        // Pull the participating channels from the appropriate variant.
        let participating = &overprint.participating;
        if participating.is_empty() {
            // RGB sources don't route to plates at all.
            return InkAction::Skip;
        }

        // Look for our target ink in the participating channels.
        if let Some(ch) = participating.iter().find(|c| c.ink == *target_ink) {
            // OPM=1 "Adobe nonzero overprint": a zero channel value on
            // DeviceCMYK means "colorant not specified" → skip.
            // §11.7.4.3 limits OPM=1 to DeviceCMYK sources; we identify
            // those by the colour variant.
            let is_cmyk = matches!(color, ResolvedColor::Cmyk { .. });
            if overprint.enabled && overprint.mode == 1 && is_cmyk && ch.value == 0.0 {
                return InkAction::Skip;
            }
            return InkAction::Paint(ch.value);
        }

        // Target ink is outside the source's colorant set. Overprint=true
        // leaves the plate untouched; overprint=false knocks it out.
        if overprint.enabled {
            InkAction::Skip
        } else {
            InkAction::Paint(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    use super::super::resolved::ParticipatingChannel;

    fn fresh_gs() -> GraphicsState {
        GraphicsState::new()
    }

    fn cmyk_color() -> ResolvedColor {
        ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.25,
            y: 0.0,
            k: 0.1,
            a: 1.0,
        }
    }

    fn cmyk_plan(enabled: bool, mode: u8) -> OverprintPlan {
        OverprintPlan {
            enabled,
            mode,
            participating: smallvec![
                ParticipatingChannel {
                    ink: InkName::new("Cyan"),
                    value: 0.5
                },
                ParticipatingChannel {
                    ink: InkName::new("Magenta"),
                    value: 0.25
                },
                ParticipatingChannel {
                    ink: InkName::new("Yellow"),
                    value: 0.0
                },
                ParticipatingChannel {
                    ink: InkName::new("Black"),
                    value: 0.1
                },
            ],
        }
    }

    #[test]
    fn cmyk_paints_named_channel() {
        let gs = fresh_gs();
        let plan = cmyk_plan(false, 0);
        let color = cmyk_color();
        let action = InkRouter::new().route(&gs, &InkName::new("Magenta"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.25));
    }

    #[test]
    fn spot_plate_outside_cmyk_knocks_out_by_default() {
        // §11.7.4 default: overprint=false → unspecified plates knock out
        // (paint 0.0 to erase underlying ink).
        let gs = fresh_gs();
        let plan = cmyk_plan(false, 0);
        let color = cmyk_color();
        let action = InkRouter::new().route(&gs, &InkName::new("PANTONE 185 C"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.0));
    }

    #[test]
    fn spot_plate_outside_cmyk_skips_when_overprint() {
        // §11.7.4 with OP=true: unspecified plates are left untouched.
        let gs = fresh_gs();
        let plan = cmyk_plan(true, 0);
        let color = cmyk_color();
        let action = InkRouter::new().route(&gs, &InkName::new("PANTONE 185 C"), &color, &plan);
        assert_eq!(action, InkAction::Skip);
    }

    #[test]
    fn opm_one_skips_zero_components_on_cmyk() {
        // §11.7.4.3 OPM=1: a zero channel on DeviceCMYK is "colorant not
        // specified" → leave the matching plate alone.
        let gs = fresh_gs();
        let plan = cmyk_plan(true, 1);
        let color = ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.0,
            y: 0.0,
            k: 0.0,
            a: 1.0,
        };
        // Plan reflects the zero values; ensure routing acts on them.
        let mut plan = plan;
        plan.participating[1].value = 0.0; // Magenta = 0
        let action = InkRouter::new().route(&gs, &InkName::new("Magenta"), &color, &plan);
        assert_eq!(action, InkAction::Skip);
    }

    #[test]
    fn opm_zero_paints_zero_components_normally() {
        // §11.7.4 OPM=0 (default): zero is *not* special — paint it
        // (which knocks the plate out at the painted pixels).
        let gs = fresh_gs();
        let mut plan = cmyk_plan(true, 0);
        plan.participating[1].value = 0.0;
        let color = ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.0,
            y: 0.0,
            k: 0.0,
            a: 1.0,
        };
        let action = InkRouter::new().route(&gs, &InkName::new("Magenta"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.0));
    }

    #[test]
    fn rgb_source_skips_all_plates() {
        // §11.7.4 doesn't define overprint for RGB sources. The plan's
        // participating set is empty (by construction in OverprintResolver),
        // so every plate gets Skip.
        let gs = fresh_gs();
        let plan = OverprintPlan {
            enabled: true,
            mode: 0,
            participating: smallvec![],
        };
        let color = ResolvedColor::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let action = InkRouter::new().route(&gs, &InkName::new("Cyan"), &color, &plan);
        assert_eq!(action, InkAction::Skip);
    }

    #[test]
    fn per_channel_devicen_routes_by_ink_name() {
        // DeviceN with named channels: route by exact ink name.
        let gs = fresh_gs();
        let plan = OverprintPlan {
            enabled: false,
            mode: 0,
            participating: smallvec![
                ParticipatingChannel {
                    ink: InkName::new("PANTONE 185 C"),
                    value: 0.75
                },
                ParticipatingChannel {
                    ink: InkName::new("Dieline"),
                    value: 0.1
                },
            ],
        };
        let color = ResolvedColor::PerChannel {
            channels: Box::new(smallvec![
                (InkName::new("PANTONE 185 C"), 0.75),
                (InkName::new("Dieline"), 0.1),
            ]),
            a: 1.0,
        };
        let action = InkRouter::new().route(&gs, &InkName::new("Dieline"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.1));
    }
}
