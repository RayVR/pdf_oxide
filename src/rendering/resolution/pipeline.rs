//! Pipeline composer — orchestrates the resolution stages.
//!
//! [`ResolutionPipeline::resolve`] runs each stage in sequence, feeding the
//! output of one into the input of the next where the data flow demands it,
//! and produces the final [`ResolvedPaintCmd`] the backend consumes.
//!
//! The order is:
//!
//! 1. **Colour** — `LogicalColor` → `ResolvedColor`. Reads `ctx` (for ICC,
//!    OutputIntent, tint-transform streams) and the intent's components.
//!    Folds in `gs.fill_alpha` / `gs.stroke_alpha` per `side`.
//! 2. **Overprint** — produces an `OverprintPlan` from `gs` + the resolved
//!    colour. Reads channel values from the colour to populate the
//!    participating-channels list.
//! 3. **Blend** — produces a `BlendPlan` from `gs.blend_mode`. Doesn't
//!    depend on colour or overprint.
//! 4. **Clip** — wraps the operator walker's composed clip mask reference
//!    into a `ClipPlan`. The composition itself is the walker's
//!    responsibility (see `apply_pending_clip` in the existing renderer);
//!    the resolver just packages the result.
//!
//! The `InkRouter` stage is not invoked here — it runs per target ink
//! inside the backend's [`super::PaintBackend::paint`] implementation for
//! per-plate backends. Composite backends don't call it at all.

use std::sync::Arc;

use crate::error::Result;

use super::blend::BlendResolver;
use super::clip::ClipResolver;
use super::color::ColorResolver;
use super::context::ResolutionContext;
use super::intent::{PaintIntent, PaintSide};
use super::overprint::OverprintResolver;
use super::resolved::ResolvedPaintCmd;

/// Composable resolution pipeline. Holds one instance of each stage.
///
/// Stages are stateless, so a single `ResolutionPipeline` can be shared
/// across all intents for all pages.
pub(crate) struct ResolutionPipeline {
    pub(crate) color: ColorResolver,
    pub(crate) overprint: OverprintResolver,
    pub(crate) blend: BlendResolver,
    pub(crate) clip: ClipResolver,
}

impl ResolutionPipeline {
    /// Build a default pipeline with every stage's stateless constructor.
    pub(crate) const fn new() -> Self {
        Self {
            color: ColorResolver::new(),
            overprint: OverprintResolver::new(),
            blend: BlendResolver::new(),
            clip: ClipResolver::new(),
        }
    }

    /// Resolve a single paint intent.
    ///
    /// `clip_mask` is the composed clip mask the operator walker maintains.
    /// We pass it through `ClipResolver` rather than reaching into the walker
    /// state directly, so the same code path works when the walker has no
    /// active clip (passes `None`).
    pub(crate) fn resolve<'a>(
        &self,
        intent: &PaintIntent<'a>,
        ctx: &ResolutionContext,
        clip_mask: Option<Arc<tiny_skia::Mask>>,
    ) -> Result<ResolvedPaintCmd<'a>> {
        let alpha = match intent.side {
            PaintSide::Fill => intent.gs.fill_alpha,
            PaintSide::Stroke => intent.gs.stroke_alpha,
        };

        let color = self.color.resolve(&intent.color, ctx, alpha)?;
        let overprint = self.overprint.resolve(intent.gs, intent.side, &color);
        let blend = self.blend.resolve(intent.gs);
        let clip = self.clip.resolve_with_mask(clip_mask);

        Ok(ResolvedPaintCmd {
            // PaintKind is `Copy` — every variant holds only borrows
            // and primitive copy types — so the memberwise copy is a
            // single dereference.
            kind: intent.kind,
            side: intent.side,
            color,
            overprint,
            blend,
            clip,
            ctm: intent.ctm,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::graphics_state::{GraphicsState, Matrix};
    use crate::object::Object;
    use smallvec::smallvec;
    use std::collections::HashMap;

    use super::super::intent::{DeviceColor, LogicalColor, PaintKind};
    use super::super::resolved::{BlendPlan, ClipPlan, ResolvedColor};
    use super::super::test_support::fixture_doc;

    fn rectangle_path() -> tiny_skia::Path {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(0.0, 0.0);
        pb.line_to(10.0, 0.0);
        pb.line_to(10.0, 10.0);
        pb.line_to(0.0, 10.0);
        pb.close();
        pb.finish().expect("non-empty path")
    }

    #[test]
    fn pipeline_resolves_device_gray_path_fill() {
        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();

        let path = rectangle_path();
        let mut gs = GraphicsState::new();
        gs.fill_alpha = 0.8;
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Fill,
            gs: &gs,
            color: LogicalColor::Device(DeviceColor::Gray(0.25)),
            ctm: Matrix::identity(),
        };

        let cmd = pipeline.resolve(&intent, &ctx, None).unwrap();

        // Colour: Gray(0.25) folded with fill_alpha=0.8 → Rgba(0.25, 0.25, 0.25, 0.8).
        match cmd.color {
            ResolvedColor::Rgba { r, g, b, a } => {
                assert!((r - 0.25).abs() < 1e-6);
                assert!((g - 0.25).abs() < 1e-6);
                assert!((b - 0.25).abs() < 1e-6);
                assert!((a - 0.8).abs() < 1e-6);
            },
            _ => panic!("expected Rgba"),
        }

        // Default GS: overprint disabled, mode 0.
        assert!(!cmd.overprint.enabled);
        assert_eq!(cmd.overprint.mode, 0);

        // Default GS blend = Normal → SourceOver native.
        match cmd.blend {
            BlendPlan::Native(tiny_skia::BlendMode::SourceOver) => {},
            other => panic!("expected SourceOver, got {other:?}"),
        }

        // No clip mask passed.
        match cmd.clip {
            ClipPlan::None => {},
            _ => panic!("expected ClipPlan::None"),
        }
    }

    #[test]
    fn pipeline_passes_through_clip_mask_arc() {
        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();
        let path = rectangle_path();
        let gs = GraphicsState::new();
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Fill,
            gs: &gs,
            color: LogicalColor::Device(DeviceColor::Gray(0.0)),
            ctm: Matrix::identity(),
        };

        let mask = Arc::new(tiny_skia::Mask::new(4, 4).unwrap());
        let cmd = pipeline.resolve(&intent, &ctx, Some(mask.clone())).unwrap();
        match cmd.clip {
            ClipPlan::Mask(m) => assert!(Arc::ptr_eq(&m, &mask)),
            _ => panic!("expected ClipPlan::Mask"),
        }
    }

    #[test]
    fn pipeline_picks_stroke_alpha_for_stroke_side() {
        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();
        let path = rectangle_path();
        let mut gs = GraphicsState::new();
        gs.fill_alpha = 0.4;
        gs.stroke_alpha = 0.6;
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Stroke,
            gs: &gs,
            color: LogicalColor::Device(DeviceColor::Rgb(1.0, 0.0, 0.0)),
            ctm: Matrix::identity(),
        };
        let cmd = pipeline.resolve(&intent, &ctx, None).unwrap();
        match cmd.color {
            ResolvedColor::Rgba { a, .. } => assert!((a - 0.6).abs() < 1e-6),
            _ => panic!("expected Rgba"),
        }
    }

    #[test]
    fn pipeline_resolves_spaced_separation_with_type4_end_to_end() {
        // Full pipeline path for the regression case: an `scn` against a
        // Separation/DeviceCMYK/Type-4 space must resolve to a non-black
        // RGBA — not the `1.0 - tint = 0` solid black the existing inline
        // path produces. This is the same logic exercised in
        // `color::tests::separation_with_type4_calculator_evaluates_program`
        // but here we run it through the whole pipeline so we also verify
        // the resolver composition (alpha fold, overprint plan, blend plan,
        // clip plan) doesn't interfere.
        let program = b"{ 0.0 exch 0.0 0.0 }";
        let mut func_dict: HashMap<String, Object> = HashMap::new();
        func_dict.insert("FunctionType".into(), Object::Integer(4));
        let func_obj = Object::Stream {
            dict: func_dict,
            data: program.to_vec().into(),
        };
        let space = Object::Array(vec![
            Object::Name("Separation".into()),
            Object::Name("MagentaSpot".into()),
            Object::Name("DeviceCMYK".into()),
            func_obj,
        ]);

        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();
        let path = rectangle_path();
        let gs = GraphicsState::new();
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Fill,
            gs: &gs,
            color: LogicalColor::Spaced {
                space: &space,
                components: smallvec![1.0],
            },
            ctm: Matrix::identity(),
        };
        let cmd = pipeline.resolve(&intent, &ctx, None).unwrap();
        match cmd.color {
            ResolvedColor::Rgba { r, g, b, a } => {
                assert!((r - 1.0).abs() < 1e-3);
                assert!((g - 0.0).abs() < 1e-3);
                assert!((b - 1.0).abs() < 1e-3);
                assert!((a - 1.0).abs() < 1e-3);
            },
            _ => panic!("expected Rgba"),
        }
    }
}
