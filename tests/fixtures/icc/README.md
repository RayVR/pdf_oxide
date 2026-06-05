# ICC profile fixtures for the OutputIntent integration suite

## Why this directory exists

`tests/test_render_output_intent.rs` needs CMYK ICC profiles that qcms
accepts at transform-build time. The existing test stub in
`tests/test_icc_cmyk_conversion.rs` is a 128-byte header-only profile
that qcms rejects (no tag table) — fine for proving the additive-clamp
fallback fires, useless for proving the OutputIntent path fires.

A freely-redistributable real production CMYK ICC profile (e.g.
CoatedFOGRA39 from the ECI press standard) would be the ideal fixture
but isn't ergonomic to commit: most are several hundred KiB and carry
licensing terms that vary by region. Apple's `Generic CMYK Profile` on
macOS is OS-bundled and not redistributable.

## Approach: in-test synthesis

`tests/test_render_output_intent.rs` synthesises a minimal valid ICC
v2 CMYK→RGB profile in code (`build_minimal_cmyk_to_rgb_lut8_profile`).
The profile carries one `A2B0` tag holding a LUT8 with a 2×2×2×2 CLUT
that maps every CMYK input to a fixed `RGB(128, 128, 128)`. That's
deliberately constant so the test pin is unambiguous: when the
OutputIntent path fires, the rendered pixel is the constant RGB the
profile encodes; when the additive-clamp fallback fires, the rendered
pixel is the §10.3.5 value (e.g. CMYK(0.25, 0, 0, 0) → RGB(191, 255,
255)).

ICC v2 profile layout follows ICC.1:2004-10:
- 128-byte header with `acsp` signature at bytes 36..40, `CMYK`
  colour-space signature at 16..20, `XYZ ` PCS at 20..24, `prtr`
  device class at 12..16, version `0x02000000` at 8..12.
- 4-byte tag count followed by tag-table entries (12 bytes each):
  signature, offset, size.
- Tag data sections, each 4-byte aligned.

The LUT8 tag (`mft1` / 0x6d667431) is the minimal interpolation table
qcms accepts for CMYK input; the LUT shape is documented in ICC §10.8.

## When to commit a binary fixture instead

If the synthesis path proves too fragile across qcms versions, swap
to a committed permissively-licensed profile. Candidates:
- ICC consortium's `srgb_v4_ICC_preference.icc` (sRGB, no good for
  CMYK).
- A small custom-built CMYK profile generated with `littlecms`
  (`cmscreate`-style tooling) and licensed under MIT / public domain.

Track which file is canonical here when the swap happens.
