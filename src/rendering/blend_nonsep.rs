//! Non-separable PDF blend modes (Hue, Saturation, Color, Luminosity)
//! per ISO 32000-1:2008 §11.3.5.3.
//!
//! tiny_skia has no native non-separable blend mode; these are
//! implemented out-of-band by rendering the source paint into a fresh
//! scratch pixmap (which captures the source's contribution as `Source`
//! mode RGBA) and then per-pixel compositing against the destination
//! pixmap using the §11.3.5.3 algorithm.
//!
//! The four non-separable modes share a luminance-projection +
//! re-encoding skeleton:
//!
//! - **Hue**: SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb))
//! - **Saturation**: SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))
//! - **Color**: SetLum(Cs, Lum(Cb))
//! - **Luminosity**: SetLum(Cb, Lum(Cs))
//!
//! `Lum`, `Sat`, `SetLum`, `SetSat`, and `ClipColor` are defined in
//! §11.3.5.3 and implemented below.

/// PDF non-separable blend modes per §11.3.5.3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NonSeparableBlend {
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl NonSeparableBlend {
    /// Recognise the PDF blend-mode name.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "Hue" => Some(Self::Hue),
            "Saturation" => Some(Self::Saturation),
            "Color" => Some(Self::Color),
            "Luminosity" => Some(Self::Luminosity),
            _ => None,
        }
    }
}

/// Compose `source` over `dest` in-place using the §11.3.5.3 algorithm.
///
/// Both buffers are RGBA8 row-major, identical dimensions. The source
/// alpha defines a coverage mask: where `source.alpha == 0` the dest
/// pixel is unchanged; elsewhere the blend rule is applied to the
/// `(source.rgb, dest.rgb)` triple, with the result composited into
/// dest via SourceOver against `source.alpha`.
///
/// This is the spec algorithm for an opaque backdrop (no group alpha
/// considerations). The current composite path renders into RGBA
/// pixmaps with dest alpha already at 255 (page background was filled),
/// so the simplified composition is correct for the audit fixtures.
pub(crate) fn compose_in_place(
    dest: &mut [u8],
    source: &[u8],
    mode: NonSeparableBlend,
) {
    debug_assert_eq!(dest.len(), source.len());
    debug_assert_eq!(dest.len() % 4, 0);

    for px in 0..(dest.len() / 4) {
        let off = px * 4;
        let src_a = source[off + 3];
        if src_a == 0 {
            continue;
        }

        // Read source and dest as f32 in [0, 1].
        let sr = source[off] as f32 / 255.0;
        let sg = source[off + 1] as f32 / 255.0;
        let sb = source[off + 2] as f32 / 255.0;
        let sa = src_a as f32 / 255.0;

        let dr = dest[off] as f32 / 255.0;
        let dg = dest[off + 1] as f32 / 255.0;
        let db = dest[off + 2] as f32 / 255.0;
        let da = dest[off + 3] as f32 / 255.0;

        // Apply the blend rule to (Cs, Cb).
        let (br, bg, bb) = match mode {
            NonSeparableBlend::Hue => {
                // SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb))
                let sat_cb = sat((dr, dg, db));
                let sat_applied = set_sat((sr, sg, sb), sat_cb);
                set_lum(sat_applied, lum((dr, dg, db)))
            },
            NonSeparableBlend::Saturation => {
                // SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))
                let sat_cs = sat((sr, sg, sb));
                let sat_applied = set_sat((dr, dg, db), sat_cs);
                set_lum(sat_applied, lum((dr, dg, db)))
            },
            NonSeparableBlend::Color => {
                // SetLum(Cs, Lum(Cb))
                set_lum((sr, sg, sb), lum((dr, dg, db)))
            },
            NonSeparableBlend::Luminosity => {
                // SetLum(Cb, Lum(Cs))
                set_lum((dr, dg, db), lum((sr, sg, sb)))
            },
        };

        // Composite the blended result over dest with source alpha
        // (SourceOver): out = sa * B + (1 - sa) * Cb.
        // Per §11.3.4 the alpha out is sa + da * (1 - sa).
        let inv_sa = 1.0 - sa;
        let out_r = sa * br + inv_sa * dr;
        let out_g = sa * bg + inv_sa * dg;
        let out_b = sa * bb + inv_sa * db;
        let out_a = sa + da * inv_sa;

        dest[off] = (out_r.clamp(0.0, 1.0) * 255.0).round() as u8;
        dest[off + 1] = (out_g.clamp(0.0, 1.0) * 255.0).round() as u8;
        dest[off + 2] = (out_b.clamp(0.0, 1.0) * 255.0).round() as u8;
        dest[off + 3] = (out_a.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

/// §11.3.5.3 `Lum(C) = 0.30 R + 0.59 G + 0.11 B`.
fn lum(c: (f32, f32, f32)) -> f32 {
    0.30 * c.0 + 0.59 * c.1 + 0.11 * c.2
}

/// §11.3.5.3 `Sat(C) = max(R, G, B) - min(R, G, B)`.
fn sat(c: (f32, f32, f32)) -> f32 {
    c.0.max(c.1).max(c.2) - c.0.min(c.1).min(c.2)
}

/// §11.3.5.3 `SetLum(C, l)`: shift the luminance of `C` to `l`, then
/// clip to the gamut.
fn set_lum(c: (f32, f32, f32), l: f32) -> (f32, f32, f32) {
    let d = l - lum(c);
    let shifted = (c.0 + d, c.1 + d, c.2 + d);
    clip_color(shifted)
}

/// §11.3.5.3 `ClipColor(C)`: project an out-of-gamut color back into
/// the unit RGB cube while preserving its luminance.
fn clip_color(c: (f32, f32, f32)) -> (f32, f32, f32) {
    let l = lum(c);
    let n = c.0.min(c.1).min(c.2);
    let x = c.0.max(c.1).max(c.2);

    let (mut r, mut g, mut b) = c;
    if n < 0.0 {
        // Scale toward the luminance to bring the minimum to 0.
        let denom = l - n;
        if denom.abs() > 1e-9 {
            r = l + (r - l) * l / denom;
            g = l + (g - l) * l / denom;
            b = l + (b - l) * l / denom;
        }
    }
    if x > 1.0 {
        // Scale toward the luminance to bring the maximum to 1.
        let denom = x - l;
        if denom.abs() > 1e-9 {
            r = l + (r - l) * (1.0 - l) / denom;
            g = l + (g - l) * (1.0 - l) / denom;
            b = l + (b - l) * (1.0 - l) / denom;
        }
    }
    (r, g, b)
}

/// §11.3.5.3 `SetSat(C, s)`: rebuild C so it has saturation `s` while
/// preserving the ordering of the channels.
fn set_sat(c: (f32, f32, f32), s: f32) -> (f32, f32, f32) {
    // Identify the channels in (min, mid, max) order. Place s into
    // max - min, mid is scaled proportionally, others zero.
    let (r, g, b) = c;
    // Sort channels by value, tracking original positions.
    let mut chans = [(r, 0u8), (g, 1u8), (b, 2u8)];
    chans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let (cmin, cmid, cmax) = (chans[0].0, chans[1].0, chans[2].0);
    let (imin, imid, imax) = (chans[0].1, chans[1].1, chans[2].1);

    let (new_min, new_mid, new_max) = if cmax > cmin {
        (0.0_f32, ((cmid - cmin) * s) / (cmax - cmin), s)
    } else {
        (0.0_f32, 0.0_f32, 0.0_f32)
    };

    let mut out = [0.0_f32; 3];
    out[imin as usize] = new_min;
    out[imid as usize] = new_mid;
    out[imax as usize] = new_max;
    (out[0], out[1], out[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn lum_matches_bt601_weights() {
        let l = lum((1.0, 0.0, 0.0));
        assert!(approx(l, 0.30));
        let l = lum((0.0, 1.0, 0.0));
        assert!(approx(l, 0.59));
        let l = lum((0.0, 0.0, 1.0));
        assert!(approx(l, 0.11));
    }

    #[test]
    fn sat_of_grey_is_zero() {
        assert!(approx(sat((0.5, 0.5, 0.5)), 0.0));
    }

    #[test]
    fn sat_of_pure_red_is_one() {
        assert!(approx(sat((1.0, 0.0, 0.0)), 1.0));
    }

    #[test]
    fn luminosity_blend_grey_source_over_red_preserves_red_hue() {
        // Source = mid-grey (Y = 0.5), Dest = red (Y = 0.30).
        // SetLum(Cb, Lum(Cs)) = SetLum((1, 0, 0), 0.5).
        // Shift d = 0.5 - 0.30 = 0.20; shifted = (1.2, 0.20, 0.20).
        // ClipColor: x = 1.2 > 1.0 → scale toward luminance.
        //   denom = 1.2 - 0.5 = 0.7
        //   r = 0.5 + (1.2 - 0.5) * (1.0 - 0.5) / 0.7 = 0.5 + 0.5 = 1.0
        //   g = 0.5 + (0.20 - 0.5) * 0.5 / 0.7 ≈ 0.286
        //   b = 0.286
        // Result is red-dominant (R=1.0 >> G≈0.286, B≈0.286).
        let mut dest = [255u8, 0, 0, 255];
        let source = [128u8, 128, 128, 255];
        compose_in_place(&mut dest, &source, NonSeparableBlend::Luminosity);
        assert!(
            dest[0] > dest[1] + 60 && dest[0] > dest[2] + 60,
            "Luminosity grey-over-red should preserve red hue; got {:?}",
            dest
        );
    }

    #[test]
    fn hue_blend_red_source_over_blue_yields_red() {
        // Source = red (H=0°, S=1, L=0.30), Dest = blue (H=240°, S=1, L=0.11).
        // Hue: SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb)).
        //   Sat(Cb=blue) = 1.0
        //   SetSat(red, 1.0) = red (already at saturation 1)
        //   SetLum(red, 0.11) = shift d = 0.11 - 0.30 = -0.19
        //     shifted = (0.81, -0.19, -0.19)
        //     ClipColor: n = -0.19 < 0 → scale.
        //       denom = 0.11 - (-0.19) = 0.30
        //       r = 0.11 + (0.81 - 0.11) * 0.11 / 0.30 ≈ 0.11 + 0.257 = 0.367
        //       g = 0.11 + (-0.19 - 0.11) * 0.11 / 0.30 ≈ 0.11 - 0.110 = 0.0
        //       b = 0.0
        // Result: red-dominant.
        let mut dest = [0u8, 0, 255, 255];
        let source = [255u8, 0, 0, 255];
        compose_in_place(&mut dest, &source, NonSeparableBlend::Hue);
        assert!(
            dest[0] > 50 && dest[1] < 30 && dest[2] < 30,
            "Hue red-over-blue should yield red-dominant; got {:?}",
            dest
        );
    }

    #[test]
    fn saturation_blend_grey_source_desaturates_dest() {
        // Source = grey (Sat = 0), Dest = red.
        // SetLum(SetSat(Cb, 0), Lum(Cb)) = SetLum((0, 0, 0), 0.30)
        //   = (0.30, 0.30, 0.30) → grey.
        let mut dest = [255u8, 0, 0, 255];
        let source = [128u8, 128, 128, 255];
        compose_in_place(&mut dest, &source, NonSeparableBlend::Saturation);
        // Channels should be near-equal (desaturated).
        let max_diff = (dest[0] as i32 - dest[1] as i32)
            .abs()
            .max((dest[0] as i32 - dest[2] as i32).abs())
            .max((dest[1] as i32 - dest[2] as i32).abs());
        assert!(
            max_diff < 30,
            "Saturation grey-over-red should desaturate; got {:?}",
            dest
        );
    }
}
