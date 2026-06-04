//! Renderer resolution pipeline — layered paint command resolution.
//!
//! # Why this module exists
//!
//! The existing renderers ([`super::page_renderer`], [`super::separation_renderer`])
//! perform colour resolution, overprint handling, blend-mode classification, and
//! clip composition **inline** at every operator's match arm. Both renderers
//! grew through copy-and-edit, so their match arms have diverged: the
//! separation renderer parses overprint state and runs per-plate ink routing
//! ([`super::separation_renderer`] `tint_for_ink`); the page renderer ignores
//! overprint entirely (`grep -c overprint src/rendering/page_renderer.rs` == 0).
//! Similarly, capability modules ([`crate::functions`] for PostScript Type 4
//! tint transforms, [`crate::color`] for ICC management, [`crate::document`]
//! `output_intent_cmyk_profile`) ship behind tests but are not consumed by the
//! composite renderer's colour path. The structural shape — capabilities at
//! the layer below the renderer, inline match arms at the renderer's layer —
//! means each new capability requires manual wiring at N match arms across
//! both renderers, and any capability that *isn't* wired manifests as a silent
//! visual bug (see the `1.0 - tint` fallback at
//! `page_renderer.rs:690` for the canonical example).
//!
//! # What this module provides
//!
//! A layered resolution pipeline that owns the conversion from "PDF logical
//! colour + graphics state" to "fully-evaluated paint command":
//!
//! ```text
//! PaintIntent          ← what the operator dispatcher emits: logical colour,
//!                        graphics-state borrow, path/glyph/image kind, clip refs
//!         ↓
//! ResolutionPipeline   ← composable stages, each with one focused method:
//!     ColorResolver        — tint transforms (Type 2, Type 4), ICCBased, Indexed,
//!                            DeviceN / Separation; consults OutputIntent + intent
//!     OverprintResolver    — per-channel overprint mask from `/OP`, `/op`, `/OPM`
//!     BlendResolver        — native tiny-skia blend mode vs. simulated
//!     ClipResolver         — composes the current clip stack into a single mask
//!     InkRouter            — per-plate routing for separation backends
//!         ↓
//! ResolvedPaintCmd     ← backend-agnostic, fully evaluated
//!         ↓
//! PaintBackend trait   ← composite (RGBA) / separation (per-plate) / future
//! ```
//!
//! # Status (this branch)
//!
//! The pipeline is wired behind an env-var toggle in the page renderer
//! (`PDF_OXIDE_RESOLUTION_PIPELINE=1`) and used for the path fill /
//! stroke / combo operators (`f`, `f*`, `S`, `s`, `B`, `B*`, `b`, `b*`).
//! With the toggle off, behaviour is byte-identical to the inline
//! dispatcher arms; toggle on, capabilities the inline arms can't reach
//! (PostScript Type 4 tint transforms on Separation/DeviceN, for one)
//! come online. Follow-up branches migrate the remaining operators
//! incrementally, validating parity at each step.
//!
//! Each stage has its own unit tests in its module: colour resolution can be
//! tested without any rendering happening, overprint resolution can be tested
//! by feeding `GraphicsState` mocks. This is the payoff of the layering —
//! capabilities become individually testable.

// Several stages (the per-plate ink router, the explicit blend planner,
// the dedicated PaintBackend trait) are scaffolding for future backends
// that have zero callers today. The `dead_code` allow keeps the module
// compiling clean under `-D warnings` while those callers come online;
// remove it once every stage has at least one production caller. The
// `unused_imports` allow covers the convenience re-exports below — the
// pilot consumes a subset (`ResolutionPipeline`, `ResolutionContext`, the
// intent + resolved types); the rest become live as follow-up branches
// migrate operators.
#![allow(dead_code, unused_imports)]

pub(crate) mod backend;
pub(crate) mod blend;
pub(crate) mod clip;
pub(crate) mod color;
pub(crate) mod context;
pub(crate) mod ink;
pub(crate) mod intent;
pub(crate) mod overprint;
pub(crate) mod pipeline;
pub(crate) mod resolved;
#[cfg(test)]
pub(crate) mod test_support;

pub(crate) use backend::PaintBackend;
pub(crate) use blend::BlendResolver;
pub(crate) use clip::ClipResolver;
pub(crate) use color::ColorResolver;
pub(crate) use context::ResolutionContext;
pub(crate) use ink::{InkAction, InkRouter};
pub(crate) use intent::{DeviceColor, LogicalColor, PaintIntent, PaintKind, PaintSide};
pub(crate) use overprint::OverprintResolver;
pub(crate) use pipeline::ResolutionPipeline;
pub(crate) use resolved::{
    BlendPlan, ClipPlan, InkName, OverprintPlan, ParticipatingChannel, ResolvedColor,
    ResolvedPaintCmd,
};
