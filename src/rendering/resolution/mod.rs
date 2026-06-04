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
//! # Design influences
//!
//! The shape of this module — operator dispatch → logical paint intent →
//! composable resolution stages → backend-agnostic resolved command →
//! pluggable backend — was driven by three public sources, named here for
//! clarity:
//!
//! 1. **ISO 32000-1:2008 (PDF 1.7)** and **ISO 32000-2:2020 (PDF 2.0)**.
//!    The layering separates spec concerns that the inline renderers had
//!    conflated:
//!      - §8.6 (colour spaces) and §8.6.6.4 (`tintTransform` for
//!        Separation/DeviceN) drive the `ColorResolver` stage.
//!      - §7.10 (functions) — Type 0 sampled, Type 2 exponential,
//!        Type 3 stitching, Type 4 PostScript calculator — drive what the
//!        resolver consults when a colour space carries a function.
//!      - §11.7.4 (overprint, `/OP`, `/op`, `/OPM`) drives `OverprintResolver`.
//!      - §11.3.5.1 / §11.3.5.2 (blend modes, separable vs. non-separable)
//!        drive `BlendResolver`.
//!      - §11.4 (transparency / soft masks / clipping) drives `ClipResolver`.
//!      - §14.11.5 (`/OutputIntents`) and §10 (colour management) drive
//!        what the resolver consults from [`crate::document`]
//!        `output_intent_cmyk_profile()` and [`crate::color`].
//!
//! 2. **Existing pdf_oxide code** that already carried the capabilities the
//!    inline renderers didn't consume:
//!      - [`crate::functions`] — PostScript calculator implementation, with
//!        Type 0/2/3/4 evaluators. Pre-dates this branch.
//!      - [`crate::color`] — qcms-based ICC pipeline. Pre-dates this branch.
//!      - [`crate::document::PdfDocument::output_intent_cmyk_profile`] —
//!        `/OutputIntents` reader. Pre-dates this branch.
//!      - [`super::ext_gstate`] `ParsedExtGState` — already parses
//!        `/OP`, `/op`, `/OPM` into typed fields; the inline page renderer
//!        was ignoring them.
//!      - [`super::separation_renderer`] `tint_for_ink` — already implements
//!        per-plate spot resolution; informed `InkRouter`'s shape.
//!      - The [`crate::content::Operator`] enum, [`super::GraphicsState`]
//!        struct, and the existing match-arm dispatch in
//!        [`super::page_renderer`] — direct input to where the pipeline
//!        slots in as the new layer between operator dispatch and the
//!        rasteriser.
//!
//! 3. **General graphics-pipeline design patterns** — the operator-dispatch /
//!    intent / resolution / backend layering is a long-standing public idiom
//!    in graphics renderers (PostScript display lists, immediate-mode → IR →
//!    backend separation in shader compilers, RIP architectures going back
//!    to the late 1980s). The module's shape lifts these public patterns into
//!    pdf_oxide; the specific stage decomposition is driven by the PDF spec
//!    sections listed above, not by any particular implementation.
//!
//! **Not consulted**: any proprietary PDF rendering engine's source, API
//! headers, or class hierarchy. The naming choices (`PaintIntent`,
//! `ResolvedPaintCmd`, `PaintBackend`, the resolver stage names) are
//! deliberately generic so as not to mirror any specific incumbent's API
//! surface.
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
pub(crate) mod separation_backend;
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
pub(crate) use separation_backend::{SeparationBackend, SeparationSurface};
