//! ICC colour-management backend abstraction.
//!
//! Two backends ship behind feature flags:
//!
//!  - [`QcmsBackend`] (`icc-qcms`, the default): Firefox's pure-Rust
//!    qcms 0.3 engine. Covers source-profile → sRGB conversion for
//!    every ICC class real PDFs ship (CMYK / RGB / Gray inputs).
//!    Cannot do CMYK → CMYK retargeting (qcms 0.3 has no CMYK output
//!    path) and silently ignores the rendering-intent parameter for
//!    CMYK sources.
//!
//!  - [`Lcms2Backend`] (`icc-lcms2`, opt-in): Little CMS via the
//!    `lcms2` crate. Press-grade — CMYK→CMYK profile retargeting
//!    through the Lab PCS, Black Point Compensation for relative-
//!    colorimetric (the press default), and rendering-intent dispatch
//!    the spec asks for. Adds a C dependency (`lcms2-sys`) so it's
//!    opt-in; consumers building for WASM or C# AOT keep the qcms
//!    default.
//!
//! At most one backend is active per build. When both features are
//! enabled, lcms2 wins — it's the strict capability superset.
//!
//! The [`IccBackend`] trait shape exists so the rest of `crate::color`
//! never imports `qcms` or `lcms2` directly: every call site goes
//! through [`Transform`](super::Transform) which is built on top of
//! [`ActiveIccBackend`]. This keeps `color.rs` free of backend cfg
//! gates and confines the qcms/lcms2 differences to this file.

use super::{IccProfile, RenderingIntent};

/// Transform-construction flags. Mirrors the lcms2 CMM's flag set; the
/// qcms backend reads only the bits it can honour and treats the rest
/// as no-ops.
#[derive(Debug, Clone, Copy, Default)]
pub struct TransformFlags {
    /// Black Point Compensation. The spec doesn't formally require BPC
    /// but the relative-colorimetric press default in real production
    /// pipelines does; without BPC, shadow tones clip to the
    /// destination's black point and the gray balance drifts. lcms2
    /// honours this bit; qcms 0.3 ignores it.
    pub black_point_compensation: bool,
}

impl TransformFlags {
    /// Convenience constructor for the press default — relative-
    /// colorimetric intent with BPC on.
    pub const fn press_default() -> Self {
        Self {
            black_point_compensation: true,
        }
    }
}

/// The trait every ICC backend implements. Two transform classes
/// matter to pdf_oxide:
///
///  - **Source → sRGB** for image / vector composite rendering. Every
///    backend supports it; the qcms 0.3 baseline only supports this.
///  - **CMYK → CMYK retargeting** for DeviceN /Process /ICCBased
///    paints whose embedded profile differs from the document
///    OutputIntent profile. Only lcms2 supports this — qcms 0.3 has
///    no CMYK output side. The retargeting flows through the Lab PCS
///    (CMYK → Lab via source AToB, Lab → CMYK via destination BToA),
///    which is the canonical press path.
///
/// Builders return `None` (rather than panic) when the backend
/// cannot construct a transform for the requested shape. Call sites
/// then fall through to the ISO 32000-1 §10.3.5 additive-clamp
/// formula or the round-5 "natural-form" reading, depending on the
/// context.
pub trait IccBackend {
    /// Backend-specific opaque source-to-sRGB transform handle.
    type SrgbTransform;
    /// Backend-specific opaque CMYK-to-CMYK retargeting transform
    /// handle.
    type CmykRetarget;

    /// Build a source-profile → sRGB transform honouring `intent`.
    /// Returns `None` when the backend can't compile the profile
    /// (malformed bytes, unsupported device class, missing tags).
    fn build_srgb_transform(
        profile: &IccProfile,
        intent: RenderingIntent,
        flags: TransformFlags,
    ) -> Option<Self::SrgbTransform>;

    /// Apply a source-to-sRGB transform to one CMYK pixel. Backends
    /// that don't support CMYK source (none currently) should return
    /// `None`. The output is byte-quantised sRGB.
    fn convert_cmyk_pixel(transform: &Self::SrgbTransform, cmyk: [u8; 4]) -> Option<[u8; 3]>;

    /// Apply a source-to-sRGB transform to a packed CMYK buffer.
    /// Output buffer length is `(input.len() / 4) * 3`.
    fn convert_cmyk_buffer(transform: &Self::SrgbTransform, cmyk: &[u8]) -> Option<Vec<u8>>;

    /// Apply a source-to-sRGB transform to a packed RGB buffer.
    /// Output buffer is the same length.
    fn convert_rgb_buffer(transform: &Self::SrgbTransform, rgb: &[u8]) -> Option<Vec<u8>>;

    /// Apply a source-to-sRGB transform to a packed grayscale buffer.
    /// Output buffer is `input.len() * 3` bytes.
    fn convert_gray_buffer(transform: &Self::SrgbTransform, gray: &[u8]) -> Option<Vec<u8>>;

    /// Build a CMYK→CMYK retargeting transform from `src_profile`
    /// (the embedded /ICCBased CMYK profile) to `dst_profile` (the
    /// document `/OutputIntents` CMYK profile) honouring `intent` and
    /// `flags`. Returns `None` when the backend can't do CMYK→CMYK
    /// (the qcms 0.3 baseline) or when profile compilation fails.
    fn build_cmyk_retarget(
        src_profile: &IccProfile,
        dst_profile: &IccProfile,
        intent: RenderingIntent,
        flags: TransformFlags,
    ) -> Option<Self::CmykRetarget>;

    /// Apply a CMYK retargeting transform to a single normalised
    /// CMYK pixel. Inputs and outputs are unit-interval f32 in the
    /// channel order C, M, Y, K. Round-tripping through 8-bit is the
    /// caller's responsibility — the trait operates in f32 so
    /// quantisation only happens at the storage boundary.
    fn retarget_cmyk_pixel(transform: &Self::CmykRetarget, cmyk: [f32; 4]) -> [f32; 4];
}

// ============================================================================
// QcmsBackend — pure-Rust default. Mirrors the surface qcms 0.3 exposes.
// ============================================================================

/// qcms-backed [`IccBackend`]. Only the source-to-sRGB methods do real
/// work; CMYK retargeting is unconditionally unsupported in qcms 0.3
/// (no CMYK output path), and that's documented as
/// `HONEST_GAP_DEVICEN_PROCESS_ICC_PROFILE_MISMATCH`.
#[cfg(feature = "icc-qcms")]
pub struct QcmsBackend;

#[cfg(feature = "icc-qcms")]
mod qcms_impl {
    use super::*;

    /// Holder so the public trait can stay backend-agnostic. The
    /// inner `qcms::Transform` is the compiled CLUT.
    pub struct SrgbTransform {
        pub(super) inner: qcms::Transform,
        pub(super) source_components: u8,
    }

    /// qcms has no CMYK→CMYK path, so the retarget transform is a
    /// permanent never-constructed marker. We use `core::convert::Infallible`
    /// as the type so it can't be instantiated at runtime — every
    /// `build_cmyk_retarget` call on `QcmsBackend` returns `None`.
    pub struct CmykRetarget(pub(super) core::convert::Infallible);

    fn qcms_intent(intent: RenderingIntent) -> qcms::Intent {
        match intent {
            RenderingIntent::Perceptual => qcms::Intent::Perceptual,
            RenderingIntent::RelativeColorimetric => qcms::Intent::RelativeColorimetric,
            RenderingIntent::Saturation => qcms::Intent::Saturation,
            RenderingIntent::AbsoluteColorimetric => qcms::Intent::AbsoluteColorimetric,
        }
    }

    impl IccBackend for QcmsBackend {
        type SrgbTransform = SrgbTransform;
        type CmykRetarget = CmykRetarget;

        fn build_srgb_transform(
            profile: &IccProfile,
            intent: RenderingIntent,
            _flags: TransformFlags,
        ) -> Option<Self::SrgbTransform> {
            let src = qcms::Profile::new_from_slice(profile.bytes(), false)?;
            let dst = qcms::Profile::new_sRGB();
            let src_ty = match profile.n_components() {
                1 => qcms::DataType::Gray8,
                3 => qcms::DataType::RGB8,
                4 => qcms::DataType::CMYK,
                _ => return None,
            };
            qcms::Transform::new_to(&src, &dst, src_ty, qcms::DataType::RGB8, qcms_intent(intent))
                .map(|inner| SrgbTransform {
                    inner,
                    source_components: profile.n_components(),
                })
        }

        fn convert_cmyk_pixel(transform: &Self::SrgbTransform, cmyk: [u8; 4]) -> Option<[u8; 3]> {
            if transform.source_components != 4 {
                return None;
            }
            let mut dst = [0u8; 3];
            transform.inner.convert(&cmyk, &mut dst);
            Some(dst)
        }

        fn convert_cmyk_buffer(transform: &Self::SrgbTransform, cmyk: &[u8]) -> Option<Vec<u8>> {
            if transform.source_components != 4 {
                return None;
            }
            let pixels = cmyk.len() / 4;
            let mut out = vec![0u8; pixels * 3];
            transform.inner.convert(cmyk, &mut out);
            Some(out)
        }

        fn convert_rgb_buffer(transform: &Self::SrgbTransform, rgb: &[u8]) -> Option<Vec<u8>> {
            if transform.source_components != 3 {
                return None;
            }
            let mut out = vec![0u8; rgb.len()];
            transform.inner.convert(rgb, &mut out);
            Some(out)
        }

        fn convert_gray_buffer(transform: &Self::SrgbTransform, gray: &[u8]) -> Option<Vec<u8>> {
            if transform.source_components != 1 {
                return None;
            }
            let mut out = vec![0u8; gray.len() * 3];
            transform.inner.convert(gray, &mut out);
            Some(out)
        }

        fn build_cmyk_retarget(
            _src_profile: &IccProfile,
            _dst_profile: &IccProfile,
            _intent: RenderingIntent,
            _flags: TransformFlags,
        ) -> Option<Self::CmykRetarget> {
            // qcms 0.3 has no CMYK output path. This is the canonical
            // "no" answer that HONEST_GAP_DEVICEN_PROCESS_ICC_PROFILE
            // _MISMATCH documents under the icc-qcms-only build. Call
            // sites fall through to the round-5 "natural form" reading
            // or the §10.3.5 additive-clamp formula.
            None
        }

        fn retarget_cmyk_pixel(transform: &Self::CmykRetarget, _cmyk: [f32; 4]) -> [f32; 4] {
            // Uninhabited: `build_cmyk_retarget` always returns None
            // on QcmsBackend, so this branch is unreachable. We match
            // on the Infallible inhabitant to teach the compiler that.
            match transform.0 {}
        }
    }
}

#[cfg(feature = "icc-qcms")]
pub use qcms_impl::{CmykRetarget as QcmsCmykRetarget, SrgbTransform as QcmsSrgbTransform};

// ============================================================================
// Lcms2Backend — Little CMS via the `lcms2` crate. Press-grade CMM.
// ============================================================================

/// lcms2-backed [`IccBackend`]. Implements the full surface including
/// CMYK→CMYK retargeting (the round-7 gap-closure path) and BPC.
#[cfg(feature = "icc-lcms2")]
pub struct Lcms2Backend;

#[cfg(feature = "icc-lcms2")]
mod lcms2_impl {
    use super::*;

    /// `Transform<u8, u8>` lets us pass `&[u8]` / `&mut [u8]` directly
    /// for every byte-packed pixel format — the lcms2 crate's "u8
    /// special case" handles the reshape internally. PixelFormat
    /// (set in `new_flags`) determines the real channel count.
    pub struct SrgbTransform {
        pub(super) inner: lcms2::Transform<u8, u8>,
        pub(super) source_components: u8,
    }

    /// CMYK→CMYK retarget. The transform is built for `CMYK_FLT`
    /// on both sides so unit-interval f32 inputs / outputs round-trip
    /// without an extra 8-bit quantisation step (caller decides when
    /// to quantise). `Transform<f32, f32>` gives us a typed surface
    /// matching the f32-CMYK pixel-format constants.
    pub struct CmykRetarget {
        pub(super) inner: lcms2::Transform<f32, f32>,
    }

    fn lcms2_intent(intent: RenderingIntent) -> lcms2::Intent {
        match intent {
            RenderingIntent::Perceptual => lcms2::Intent::Perceptual,
            RenderingIntent::RelativeColorimetric => lcms2::Intent::RelativeColorimetric,
            RenderingIntent::Saturation => lcms2::Intent::Saturation,
            RenderingIntent::AbsoluteColorimetric => lcms2::Intent::AbsoluteColorimetric,
        }
    }

    fn lcms2_flags(flags: TransformFlags) -> lcms2::Flags {
        let base = lcms2::Flags::default();
        if flags.black_point_compensation {
            base | lcms2::Flags::BLACKPOINT_COMPENSATION
        } else {
            base
        }
    }

    fn src_pixel_format(n_components: u8) -> Option<lcms2::PixelFormat> {
        match n_components {
            1 => Some(lcms2::PixelFormat::GRAY_8),
            3 => Some(lcms2::PixelFormat::RGB_8),
            4 => Some(lcms2::PixelFormat::CMYK_8),
            _ => None,
        }
    }

    impl IccBackend for Lcms2Backend {
        type SrgbTransform = SrgbTransform;
        type CmykRetarget = CmykRetarget;

        fn build_srgb_transform(
            profile: &IccProfile,
            intent: RenderingIntent,
            flags: TransformFlags,
        ) -> Option<Self::SrgbTransform> {
            let src = lcms2::Profile::new_icc(profile.bytes()).ok()?;
            let dst = lcms2::Profile::new_srgb();
            let in_fmt = src_pixel_format(profile.n_components())?;
            let out_fmt = lcms2::PixelFormat::RGB_8;
            let inner = lcms2::Transform::new_flags(
                &src,
                in_fmt,
                &dst,
                out_fmt,
                lcms2_intent(intent),
                lcms2_flags(flags),
            )
            .ok()?;
            Some(SrgbTransform {
                inner,
                source_components: profile.n_components(),
            })
        }

        fn convert_cmyk_pixel(transform: &Self::SrgbTransform, cmyk: [u8; 4]) -> Option<[u8; 3]> {
            if transform.source_components != 4 {
                return None;
            }
            let src: [u8; 4] = cmyk;
            let mut dst = [0u8; 3];
            transform.inner.transform_pixels(&src, &mut dst);
            Some(dst)
        }

        fn convert_cmyk_buffer(transform: &Self::SrgbTransform, cmyk: &[u8]) -> Option<Vec<u8>> {
            if transform.source_components != 4 {
                return None;
            }
            let pixels = cmyk.len() / 4;
            let mut out = vec![0u8; pixels * 3];
            transform.inner.transform_pixels(cmyk, &mut out);
            Some(out)
        }

        fn convert_rgb_buffer(transform: &Self::SrgbTransform, rgb: &[u8]) -> Option<Vec<u8>> {
            if transform.source_components != 3 {
                return None;
            }
            let mut out = vec![0u8; rgb.len()];
            transform.inner.transform_pixels(rgb, &mut out);
            Some(out)
        }

        fn convert_gray_buffer(transform: &Self::SrgbTransform, gray: &[u8]) -> Option<Vec<u8>> {
            if transform.source_components != 1 {
                return None;
            }
            let mut out = vec![0u8; gray.len() * 3];
            transform.inner.transform_pixels(gray, &mut out);
            Some(out)
        }

        fn build_cmyk_retarget(
            src_profile: &IccProfile,
            dst_profile: &IccProfile,
            intent: RenderingIntent,
            flags: TransformFlags,
        ) -> Option<Self::CmykRetarget> {
            // Both sides must be CMYK by construction. Caller is
            // responsible for that pre-check; we bail anyway if the
            // profile header disagrees.
            if src_profile.n_components() != 4 || dst_profile.n_components() != 4 {
                return None;
            }
            let src = lcms2::Profile::new_icc(src_profile.bytes()).ok()?;
            let dst = lcms2::Profile::new_icc(dst_profile.bytes()).ok()?;
            // Both sides must advertise CmykData — a printer-class
            // profile that secretly emits LabData would otherwise
            // silently produce garbage.
            if !matches!(src.color_space(), lcms2::ColorSpaceSignature::CmykData) {
                return None;
            }
            if !matches!(dst.color_space(), lcms2::ColorSpaceSignature::CmykData) {
                return None;
            }
            let inner = lcms2::Transform::new_flags(
                &src,
                lcms2::PixelFormat::CMYK_FLT,
                &dst,
                lcms2::PixelFormat::CMYK_FLT,
                lcms2_intent(intent),
                lcms2_flags(flags),
            )
            .ok()?;
            Some(CmykRetarget { inner })
        }

        fn retarget_cmyk_pixel(transform: &Self::CmykRetarget, cmyk: [f32; 4]) -> [f32; 4] {
            let src: [f32; 4] = cmyk;
            let mut dst = [0f32; 4];
            transform.inner.transform_pixels(&src, &mut dst);
            dst
        }
    }
}

#[cfg(feature = "icc-lcms2")]
pub use lcms2_impl::{CmykRetarget as Lcms2CmykRetarget, SrgbTransform as Lcms2SrgbTransform};

// ============================================================================
// NoOpBackend — fallback when neither icc-qcms nor icc-lcms2 is enabled.
// ============================================================================

/// No-CMM backend. Every `build_*` returns `None` so call sites in
/// [`crate::color::Transform`] fall straight through to the §10.3.5
/// additive-clamp formula. This is the path WASM / C# AOT consumers
/// hit when they build with `--no-default-features` and don't opt
/// into either ICC feature.
#[cfg(not(any(feature = "icc-qcms", feature = "icc-lcms2")))]
pub struct NoOpBackend;

#[cfg(not(any(feature = "icc-qcms", feature = "icc-lcms2")))]
mod noop_impl {
    use super::*;

    /// Uninhabited — the `NoOpBackend` never constructs one of these.
    pub struct SrgbTransform(pub(super) core::convert::Infallible);
    /// Uninhabited — the `NoOpBackend` never constructs one of these.
    pub struct CmykRetarget(pub(super) core::convert::Infallible);

    impl IccBackend for NoOpBackend {
        type SrgbTransform = SrgbTransform;
        type CmykRetarget = CmykRetarget;

        fn build_srgb_transform(
            _profile: &IccProfile,
            _intent: RenderingIntent,
            _flags: TransformFlags,
        ) -> Option<Self::SrgbTransform> {
            None
        }
        fn convert_cmyk_pixel(transform: &Self::SrgbTransform, _cmyk: [u8; 4]) -> Option<[u8; 3]> {
            match transform.0 {}
        }
        fn convert_cmyk_buffer(transform: &Self::SrgbTransform, _cmyk: &[u8]) -> Option<Vec<u8>> {
            match transform.0 {}
        }
        fn convert_rgb_buffer(transform: &Self::SrgbTransform, _rgb: &[u8]) -> Option<Vec<u8>> {
            match transform.0 {}
        }
        fn convert_gray_buffer(transform: &Self::SrgbTransform, _gray: &[u8]) -> Option<Vec<u8>> {
            match transform.0 {}
        }
        fn build_cmyk_retarget(
            _src_profile: &IccProfile,
            _dst_profile: &IccProfile,
            _intent: RenderingIntent,
            _flags: TransformFlags,
        ) -> Option<Self::CmykRetarget> {
            None
        }
        fn retarget_cmyk_pixel(transform: &Self::CmykRetarget, _cmyk: [f32; 4]) -> [f32; 4] {
            match transform.0 {}
        }
    }
}

#[cfg(not(any(feature = "icc-qcms", feature = "icc-lcms2")))]
pub use noop_impl::{CmykRetarget as NoOpCmykRetarget, SrgbTransform as NoOpSrgbTransform};

// ============================================================================
// ActiveIccBackend — compile-time selection. lcms2 wins when both are on.
// ============================================================================

// ActiveIccBackend: the backend the rest of `crate::color` dispatches
// through. Resolved at compile time from the feature flag combination:
//   icc-lcms2                          → Lcms2Backend
//   icc-qcms (and not icc-lcms2)      → QcmsBackend
//   neither                            → NoOpBackend

/// Active ICC backend (compile-time selected — see module docs).
#[cfg(feature = "icc-lcms2")]
pub type ActiveIccBackend = Lcms2Backend;

/// Active ICC backend (compile-time selected — see module docs).
#[cfg(all(feature = "icc-qcms", not(feature = "icc-lcms2")))]
pub type ActiveIccBackend = QcmsBackend;

/// Active ICC backend (compile-time selected — see module docs).
#[cfg(not(any(feature = "icc-qcms", feature = "icc-lcms2")))]
pub type ActiveIccBackend = NoOpBackend;

/// Backend-name diagnostic for `Debug` output and the
/// `BACKEND_NAME` reporting hook the round-7 probes consume.
pub const fn active_backend_name() -> &'static str {
    #[cfg(feature = "icc-lcms2")]
    {
        "lcms2"
    }
    #[cfg(all(feature = "icc-qcms", not(feature = "icc-lcms2")))]
    {
        "qcms"
    }
    #[cfg(not(any(feature = "icc-qcms", feature = "icc-lcms2")))]
    {
        "noop"
    }
}
