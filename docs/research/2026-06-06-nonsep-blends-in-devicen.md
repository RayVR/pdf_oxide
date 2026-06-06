# Non-separable blend modes in a DeviceN compositing space

Research note — pdf_oxide issue #46 (SMask in separation renderer, composite-then-separate path)

Date: 2026-06-06
Status: research only — gates the design+impl brief

---

## 1. Executive summary

The PDF specification **forbids `DeviceN` as a blending colour space**. ISO 32000-1:2008 §11.3.4 enumerates the legal blending colour spaces and `DeviceN` is explicitly excluded; ISO 32000-2:2020 carries the same restriction forward in §11.4.5 / §11.6.6 (the group-attributes `CS` entry). The spec also says spot colorants "shall not be subject to conversion to or from the colour space of the enclosing transparency group" (§11.7.3) — they ride alongside the process-colour blend space in a parallel sidecar plane and are blended **component-by-component** with the corresponding component of the backdrop.

Consequently the question "how do non-separable blends work in an N-channel DeviceN blend space" is essentially malformed at the spec level: it cannot happen for a conforming document. The architecturally sound answer for pdf_oxide is **approach (B), restricted further**:

- The actual blending colour space is **3 or 4 process-colour components** (`DeviceGray`, `DeviceRGB`, `DeviceCMYK`, the CIE-based equivalents, or bidirectional `ICCBased` of N∈{1,3,4}). Non-separable blend modes run on those process components, using the §11.3.5.3 RGB formulas (with the CMYK adjustment in §11.3.5.3 / Table 137 for the `K` channel).
- The sidecar spot planes are blended **per-component, separably**, regardless of what blend mode the graphics-state `BM` parameter names — because §11.7.4.2 mandates: *"only separable, white-preserving blend modes shall be used for spot colours. If the specified blend mode is not separable and white-preserving, it shall apply only to process colour components, and the **Normal** blend mode shall be substituted for spot colours."* `Hue`, `Saturation`, `Color`, `Luminosity` are all non-separable, so the spot lanes simply run `Normal`.

That is the spec-correct answer. No HSL projection from an N-channel vector. No invented luma weights for spot inks. No fallback-to-Normal for the whole object.

---

## 2. Spec citations

### 2.1 ISO 32000-1:2008 (PDF 1.7)

All quotations below are paraphrases or short quotations from the local copy of ISO 32000-1:2008 in `docs/spec/pdf.md`. Line numbers refer to that file for reproducibility.

**§11.3.4 "Blending Colour Space"** — the closed list of legal blend spaces (lines 22011–22037):

> "Of the PDF colour spaces described in Section 8.6, the following shall be supported as blending colour spaces:
>  - **DeviceGray**
>  - **DeviceRGB**
>  - **DeviceCMYK**
>  - **CalGray**
>  - **CalRGB**
>  - **ICCBased** colour spaces equivalent to the preceding (including calibrated _CMYK_)
>
> The **Lab** space and **ICCBased** spaces that represent lightness and chromaticity separately … shall not be used as blending colour spaces … In addition, an **ICCBased** space used as a blending colour space shall be bidirectional."

`DeviceN` and `Separation` are conspicuously absent. They are confirmed-absent immediately afterward (lines 22040–22044):

> "The blending colour space shall be consulted only for process colours. Although blending may also be done on individual spot colours specified in a **Separation** or **DeviceN** colour space, such colours shall not be converted to a blending colour space (except in the case where they first revert to their alternate colour space …). Instead, the specified colour components shall be blended individually with the corresponding components of the backdrop."

This is the **single most important sentence** for the architecture. Spot/DeviceN components blend "individually with the corresponding components of the backdrop" — that is the textbook description of a **separable, per-component** operation.

**§11.3.5 "Blend Mode"** — separable vs. non-separable definition (line 22078):

> "A blend mode is termed _separable_ if each component of the result colour is completely determined by the corresponding components of the constituent backdrop and source colours … A separable blend mode may be used with any colour space, since it applies independently to any number of components. **Only separable blend modes shall be used for blending spot colours.**" (Emphasis added; lines 22086–22089.)

**§11.3.5.3 "Non-separable blend modes"** — applicability and CMYK adjustment (lines 22168–22189, 22442–22452):

> "Table 137 lists the standard nonseparable blend modes. Since the nonseparable blend modes consider all colour components in combination, their computation depends on the blending colour space in which the components are interpreted. They may be applied to all multiple-component colour spaces that are allowed as blending colour spaces (see 'Blending Colour Space')."

The phrase "allowed as blending colour spaces" is load-bearing. It points back to the §11.3.4 list, which does not contain `DeviceN`. The text continues:

> "The nonseparable blend mode formulas make use of several auxiliary functions. These functions operate on colours that are assumed to have red, green, and blue components. Blending of _CMYK_ colour spaces requires special treatment, as described in this sub-clause."

The non-sep formulas are **definitionally 3-component**. CMYK is handled by an explicit projection:

> "Blending in _CMYK_ spaces (including both **DeviceCMYK** and **ICCBased** calibrated _CMYK_ spaces) shall be handled in the following way:
>  - The _C_, _M_, and _Y_ components shall be converted to their complementary _R_, _G_, and _B_ components in the usual way. The preceding formulas shall be applied to the _RGB_ colour values. The results shall be converted back to _C_, _M_, and _Y_.
>  - For the _K_ component, the result shall be the _K_ component of _C_<sub>b</sub> for the **Hue**, **Saturation**, and **Color** blend modes; it shall be the _K_ component of _C_<sub>s</sub> for the **Luminosity** blend mode."

The auxiliary functions `Lum`, `Sat`, `SetLum`, `SetSat`, `ClipColor` are defined only over the 3-vector `(C.red, C.green, C.blue)`. The BT.601-style weights are pinned:

> `Lum(C) = 0.3 × C.red + 0.59 × C.green + 0.11 × C.blue`
> `Sat(C) = max(C.red, C.green, C.blue) - min(C.red, C.green, C.blue)`

**§11.6.3 "Specifying Blending Colour Space and Blend Mode"** (lines 23720–23721):

> "The current blend mode shall always apply to process colour components; but only sometimes may apply to spot colorants, see 11.7.4.2, 'Blend Modes and Overprinting,' for details."

**§11.6.6 "Transparency Group XObjects" / Table 147 `/CS` entry** (line 24064): the group colour space "shall be any device or CIE-based colour space that treats its components as independent additive or subtractive values in the range 0.0 to 1.0, subject to the restrictions described in 11.3.4, 'Blending Colour Space.' **These restrictions exclude Lab and lightness-chromaticity ICCBased colour spaces, as well as the special colour spaces Pattern, Indexed, Separation, and DeviceN.**"

This is the **second authoritative exclusion** of `DeviceN` as a blend / group space, and it is unambiguous.

**§11.7.3 "Spot Colours and Transparency"** (lines 24341–24368). The model is the sidecar:

> "When an object is painted transparently with a spot colour component that is available in the output device, that colour shall be composited with the corresponding spot colour component of the backdrop, independently of the compositing that is performed for process colours. A spot colour retains its own identity; it shall not be subject to conversion to or from the colour space of the enclosing transparency group or page."

And on how missing components are filled (line 24362):

> "Only a single shape value and opacity value shall be maintained at each point in the computed group results; they shall apply to both process and spot colour components. In effect, every object shall be considered to paint every existing colour component, both process and spot. Where no value has been explicitly specified for a given component in a given object, an additive value of 1.0 (or a subtractive tint value of 0.0) shall be assumed."

**§11.7.4.2 "Blend Modes and Overprinting"** (lines 24483–24489) — the binding rule for non-sep blends and spot lanes:

> "The PDF graphics state specifies only one current blend mode parameter, which shall always apply to process colorants and sometimes to spot colorants as well. **Specifically, only separable, white-preserving blend modes shall be used for spot colours. If the specified blend mode is not separable and white-preserving, it shall apply only to process colour components, and the Normal blend mode shall be substituted for spot colours.**" (Emphasis added.)

This is the **dispositive citation**. The four non-sep modes (`Hue`, `Saturation`, `Color`, `Luminosity`) are non-separable, therefore they are forbidden on spot channels by name, and the spec instructs the conforming reader to substitute `Normal` on the spot lanes. Note also: among the standard separable modes only `Difference` and `Exclusion` are not white-preserving (line 22492 / Note 2), so they too fall back to `Normal` on spot channels.

**Annex G** — ISO 32000-1 Annex G covers Linearized PDF, not transparency examples. The original Adobe PDF Reference 1.7 had an annex with worked transparency examples that was dropped in the ISO redaction. None of the surviving worked examples in §11 involve DeviceN as a blend space (which is consistent with it being forbidden).

### 2.2 ISO 32000-2:2020 (PDF 2.0)

PDF 2.0 reorganises Clause 11 but **preserves the exclusion**. The pdfa.org errata page for clause 11 (a public errata mirror against ISO 32000-2:2020) summarises the rule as: "any device or CIE-based colour space that treats its components as independent additive or subtractive values" with the exclusion list "Lab color spaces, lightness-chromaticity ICCBased color spaces, Pattern, Indexed, Separation, DeviceN." The §11.3.5 non-sep formulas, BT.601 weights and CMYK `K`-channel rule are also retained verbatim in PDF 2.0 (confirmed via multiple secondary descriptions of the §11.3.5 text). No PDF 2.0-specific clarification was found that loosens or tightens the DeviceN rule for non-sep blends — because the case is structurally impossible: DeviceN is not allowed as the blend space in the first place.

### 2.3 ISO 15930-7 (PDF/X-4)

PDF/X-4 permits live transparency over spot-bearing artwork, but it does **not** redefine the §11 transparency model. It constrains the OutputIntent and the relationship between the page group's blend space and the device, but the blend space itself is still drawn from the §11.3.4 list. PDF/X-4 therefore inherits the §11.7.4.2 "Normal-on-spots-for-non-sep" rule. The PDF/X-4 standard ISO 15930-7:2008 / ISO 15930-7:2010 (against PDF 1.6) is the operative version of the standard; nothing in its scope statement contradicts §11.7.4.2.

### 2.4 W3C Compositing and Blending Level 1

The W3C non-sep formulas are mathematically identical to PDF's §11.3.5.3 (BT.601 weights, identical `Lum/Sat/SetLum/SetSat/ClipColor` definitions). The W3C spec explicitly restricts itself to RGB and does **not** generalise to N>3-channel blend spaces. This is consistent with the PDF position.

### 2.5 Adobe historical reference

The 2006 PDF Reference 1.6 blend-modes addendum (printtechnologies.org host) introduced the four non-sep modes in their current form. The historical Adobe transparency tech note (the basis for §11) similarly assumes a 3-component perceptual projection. The point is the same: non-sep blends are intrinsically 3-component; the spec never describes an N-channel extension.

---

## 3. Approach evaluation

The question prompts five approaches. Each is graded against (1) spec defensibility, (2) prepress correctness, (3) tractability for pdf_oxide. The grading reflects the §11.7.4.2 rule above.

### (A) Project to 3-component perceptual, blend, project back across all N

- **Spec defensibility:** poor. The spec does **not** describe this projection for spot lanes. §11.7.3 forbids converting spots out of their identity into the blend space; this approach does exactly that.
- **Prepress correctness:** poor. The forward projection demands a tint-transform-based combine of `CMYK + spots → RGB/Lab` which is well-defined per the alternate colour space. The **inverse** (`RGB/Lab → CMYK + spots`) is undefined without a device-link profile. Spot inks lose identity through the round trip.
- **Tractability:** poor. Implementing the inverse map at compositing time is impractical and not what any prepress workflow does.

### (B) Apply blend to process channels (CMYK) only; pass spots through with `Normal`

- **Spec defensibility:** **high — directly endorsed by §11.7.4.2.** Quotation: "*If the specified blend mode is not separable and white-preserving, it shall apply only to process colour components, and the Normal blend mode shall be substituted for spot colours.*" That is exactly approach (B).
- **Prepress correctness:** high. The non-sep blend is a perceptual operation on a 3-component perceptual signal. Spot inks have no fixed perceptual contribution without their tint transform, and the tint transform is a device-fallback path that is irrelevant if the spot ink itself is available on the press. Carrying the spot lane through with `Normal` preserves spot-ink identity exactly as the §11.7.3 sidecar model intends.
- **Tractability:** high. The page renderer already wires non-sep formulas for `DeviceCMYK` per §11.3.5.3; the sidecar lanes just need a Normal path. Knockout / isolation logic is unchanged.

### (C) Extend `Lum/Sat` to N channels with invented weights

- **Spec defensibility:** none. The spec defines `Lum` and `Sat` over exactly `(red, green, blue)` and gives a single CMYK extension (RGB-complement for `C,M,Y`; rule-of-thumb for `K`). It never defines weights for spot lanes. Any weighting choice is invented.
- **Prepress correctness:** undefined. The result depends on the spot ink's actual spectral properties, which a renderer does not know. Any weighting choice will produce results that no other tool will match.
- **Tractability:** medium-low. Easy to code but impossible to defend.

### (D) Forbid non-sep blends in DeviceN blend space; force fallback to `Normal`

This is partially correct, but **too aggressive**: it would drop the non-sep behaviour on the **process lanes too**. The spec's actual instruction (§11.7.4.2) is finer-grained — only the spot lanes fall back to `Normal`; the process lanes still run the requested non-sep formula. (D) collapses into the right answer only if the entire blend space were DeviceN, which the spec forbids upstream. So (D) is moot in practice: by the time a non-sep blend executes, the blend space is already 3- or 4-component process.

### (E) Anything else

The only other "approach" with any literature support is "the document is non-conforming, refuse to render" — i.e. preflight-style rejection. That is appropriate for a hard prepress pipeline but not for a permissive renderer. It does not change the math; it just gates input.

---

## 4. Recommendation

**Adopt approach (B), tightened to the exact §11.7.4.2 rule.**

The architectural shape is the same DeviceN-extended sidecar plane that issue #46 already describes:

```
Compositing buffer = (process_lanes[N_process], spot_lanes[N_spot], shape, opacity)
where N_process ∈ {1, 3, 4}  (Gray | RGB | CMYK group colour space)
      N_spot    = number of active spot inks present in the job
```

The compositing pseudocode for a single transparent paint becomes:

```
for each pixel (x,y):
    cs_p, cs_s = source_process_components, source_spot_components   # both vectors
    cb_p, cb_s = backdrop_process_components, backdrop_spot_components

    process_blend = blend_function_of_BM_modulated_for_separability(cs_p, cb_p)
    # if BM is separable: apply per-component
    # if BM is one of Hue/Saturation/Color/Luminosity: apply the §11.3.5.3
    #   RGB formulas (with CMYK K-channel rule if N_process == 4).

    if BM is separable AND BM is white-preserving:
        spot_blend = blend_function(cs_s, cb_s)   # component-wise
    else:
        spot_blend = cs_s                          # Normal on the spot lanes

    # Then the §11.3.3 standard compositing formula applies, per-component,
    # to both process_blend and spot_blend, using shape/opacity from the
    # graphics state.
```

Concretely:

- `Hue`, `Saturation`, `Color`, `Luminosity` → process lanes use the §11.3.5.3 formulas; spot lanes substitute `Normal`.
- `Difference`, `Exclusion` → process lanes use the listed separable formula (these are separable but **not** white-preserving); spot lanes substitute `Normal`.
- `Normal`, `Multiply`, `Screen`, `Overlay`, `Darken`, `Lighten`, `ColorDodge`, `ColorBurn`, `HardLight`, `SoftLight` → separable and white-preserving; spot lanes use the same formula component-wise.

If the group colour space is `DeviceGray`, the §11.3.5.3 formulas collapse trivially (single component blended against itself; non-sep modes are degenerate but well-defined since `Lum` and `Sat` over a 1-vector are `c` and `0` respectively, so `Hue` becomes "backdrop", `Luminosity` becomes "source", and `Color`/`Saturation` become identity — these reduce to the same end-state the spec produces via the §11.3.5.3 CMYK projection if you contract `C=M=Y=0`).

### Why this is "composite-then-separate" rather than "separate-then-composite"

§11.7.3 and §11.7.4.2 together mandate the composite-then-separate ordering: the compositing buffer carries process and spot lanes side-by-side, all blends evaluate against that buffer, and only after every transparency / SMask / knockout operation has been resolved do we hand off to the per-plate output writer (which is the second stage — §11.6.7 / Annex G of the original Adobe transparency model — and what pdf_oxide already does for separation rendering).

The DeviceN-extended sidecar is *not* a DeviceN **blend space**. It is a 3-or-4-component **process blend space** with one extra register per active spot ink, and the spot registers do not see non-sep math. The spec's "DeviceN cannot be a blend space" rule is therefore not violated — DeviceN is the **output** colour model of the final plane stack, not the **blend** colour space.

### Behavioural pin points (for the implementation brief)

1. The process lanes' non-sep math uses BT.601 weights pinned to `(0.30, 0.59, 0.11)`. pdf_oxide already pins these (task #51).
2. When `N_process == 4` (CMYK group), apply the §11.3.5.3 CMYK adjustment: complement `CMY → RGB`, blend, complement back to `CMY`; the `K` channel uses `K_b` for Hue/Saturation/Color and `K_s` for Luminosity. No invented combine of `K` and `(R,G,B)`.
3. When `N_process` is CIE-based (CalRGB / ICCBased-RGB / ICCBased-CMYK / CalGray / ICCBased-Gray), the §11.3.5.3 formulas apply directly to the device-space components (the colour space is treated as if it were `DeviceRGB`-like for the purpose of the blend math; §11.7.2 notes the result is then interpreted in that CIE-based space). The CMYK projection rule still applies for ICCBased-CMYK because the spec says so explicitly: "Blending in _CMYK_ spaces (including both **DeviceCMYK** and **ICCBased** calibrated _CMYK_ spaces)".
4. Spot lanes always substitute `Normal` for non-sep BM, and substitute `Normal` for `Difference`/`Exclusion`. They use the requested BM only for separable white-preserving modes.
5. Soft masks: §11.6.5.2 already forbids spot lanes inside a soft-mask group's `G` stream — they revert to the alternate colour space. So when the SMask is computed, its blend space is process-only and the question doesn't arise.

### What pdf_oxide does **not** need to do

- It does not need to define `Lum`/`Sat` over an N-channel vector. The spec never asks for this.
- It does not need to invert a perceptual→device map across the spot dimension. The spec never asks for this either.
- It does not need to fall back to `Normal` on the process lanes when a spot lane is present. The §11.7.4.2 rule splits the BM per lane class; the process lanes always honour the requested BM.

---

## 5. Edge cases for QA

These are the fixtures that would expose a wrong implementation. They are the test scenarios task #46 / #51 should pin.

1. **Pure-spot source over CMYK backdrop with `/BM /Luminosity`.** Backdrop = (40%C, 0,0,0, 0%spot). Source paints only the spot channel at 80% with `Luminosity`. Expected: process lanes unchanged (because `Luminosity` is non-sep → `Normal` on spot, but the source has no process components, so `Cs_p ≡ (0,0,0,0)` additive `(1,1,1,1)`; under `Normal` over an opaque process backdrop the additive 1.0 source leaves backdrop unchanged after the §11.7.4.2 / Table 149 rule); spot lane gets 80% via `Normal`. **Verifies:** non-sep mode does not corrupt either lane class.

2. **CMYK source + CMYK backdrop with `/BM /Color` and one active spot lane on the page.** Source = (10%C, 90%M, 50%Y, 30%K, 0%spot). Backdrop = (60%C, 0%M, 40%Y, 20%K, 50%spot). Expected: process lanes per §11.3.5.3 CMYK projection (complement, RGB blend, complement back; `K = K_b = 20%`); spot lane runs `Normal`, which for source 0% / additive 1.0 leaves the backdrop's 50% spot unchanged. **Verifies:** CMYK K-channel rule on Color/Saturation/Hue uses backdrop K, and spot lane is not perturbed by the non-sep formula.

3. **CMYK source + CMYK backdrop with `/BM /Luminosity` and one active spot lane.** Same as (2) but `Luminosity`. Expected: process lanes per §11.3.5.3 with `K = K_s = 30%`; spot lane again `Normal` → unchanged. **Verifies:** the Luminosity K-channel rule (uses source K, opposite of Hue/Saturation/Color).

4. **Mixed source: CMYK + spot in a single DeviceN paint with `/BM /Hue`.** Source = (20%C, 60%M, 0%Y, 0%K, 70%spot). Backdrop = (50%C, 50%M, 50%Y, 10%K, 0%spot). Expected: process lanes execute `Hue` per the RGB projection (K = K_b = 10%); spot lane gets the source 70% via `Normal` (i.e. the spot channel is *painted* — not blended via Hue). **Verifies:** the per-lane BM split.

5. **Non-isolated group with `/BM /Difference` (separable but not white-preserving) over CMYK + spot backdrop.** Source paints all process lanes and one spot lane. Expected: process lanes use the Difference formula component-wise; spot lane substitutes `Normal` (the §11.7.4.2 rule covers both non-separable *and* non-white-preserving). **Verifies:** the rule does not collapse to "non-sep only" — it also catches Difference/Exclusion for spots.

6. **Soft-mask `/S /Luminosity` whose group `G` content stream references a DeviceN colour.** Per §11.6.5.2, the spot components are unavailable inside the mask group's content stream — the alternate colour space substitutes. The mask group is then composited against `BC` in its own (3-or-4-component) CS, and the luminosity is extracted to drive the mask. **Verifies:** the SMask-group path never reaches the N-channel sidecar.

7. **Non-conforming input: a transparency group XObject declaring `/CS [/DeviceN [/Cyan /Magenta /Yellow /Black /PANTONE 185 C] /DeviceCMYK <tint transform>]`.** This violates §11.3.4 / §11.6.6's `CS` rule. A conforming reader can either (a) reject the group (preflight stance) or (b) substitute the alternate colour space (the spec describes this fallback for DeviceN paints, and applying it consistently to a DeviceN group attempt is the most permissive defensible move). Pdf_oxide should pick one stance and document it as an `HONEST_GAP` (see §6 below). **Verifies:** the renderer doesn't silently invent N-channel HSL math when a malformed file requests it.

---

## 6. Open questions / `HONEST_GAP_*` candidates

These are points where the spec genuinely does not give an answer and the implementation has to make a defensible choice we should pin in code with a comment.

1. **`HONEST_GAP_NONSEP_DEVICEN_GROUP`** — what to do if a (non-conforming) document declares a transparency group with `/CS /DeviceN`. The spec forbids this but does not specify reader behaviour. Defensible options:
   - Reject the file as non-conforming (preflight-grade RIPs do this).
   - Substitute the alternate colour space declared in the DeviceN object (most permissive; matches how DeviceN paint operators reduce to their alternate when the colorant isn't available).
   - Force the group `CS` to the inherited parent group's CS (least-surprising for downstream consumers).
   pdf_oxide should pick option 2 (substitute alternate) for consistency with how DeviceN is handled for paint operators, and emit a parse-time warning. This needs a one-line decision in the design+impl brief.

2. **`HONEST_GAP_NONSEP_GRAY_DEGENERATE`** — the §11.3.5.3 non-sep formulas have well-defined but degenerate behaviour over a `DeviceGray` blend space (`Sat` is identically 0, so `Hue` collapses to the backdrop and `Saturation` and `Color` collapse to `SetLum(C_x, Lum(C_b))` which for a 1-vector reduces to `C_x` after clipping). The spec does not call this out. pdf_oxide should encode the degenerate behaviour explicitly and add a comment citing §11.3.5.3 + §11.3.4 so the reader understands it is not a stub.

3. **`HONEST_GAP_NONSEP_K_CHANNEL_FOR_NON_CMYK_FOUR_COMPONENT_ICC`** — the spec's CMYK rule for the `K` channel applies to "**DeviceCMYK** and **ICCBased** calibrated _CMYK_". A 4-component **non-CMYK** ICCBased profile (e.g. a `n=4` Lab-derived profile, or a 4-ink Hexachrome-style ICCBased space deployed as a working space) is allowed by §11.3.4 only if its components are independent additive/subtractive. The spec does not say what to do for non-sep blends in such a space. Defensible: treat as if the channels were `(R, G, B, K-like fourth)` with the K-rule applied to component index 3. pdf_oxide should pin this in code with a citation; in practice this case is vanishingly rare.

4. **`HONEST_GAP_NONSEP_BIDIRECTIONAL_ICC_REQUIRED`** — §11.3.4 says ICCBased blend spaces must contain both `AToB` and `BToA` transformations. pdf_oxide should reject a group whose declared `CS` is unidirectional `ICCBased`, or silently fall back to `DeviceCMYK`/`DeviceRGB` per the profile's component count. We have an existing OutputIntent code path; this is mostly a matter of plumbing the check.

5. **PDF 2.0 wording check** — the report above is based on PDF 1.7 text + a PDF 2.0 errata mirror confirming the same rule. The team should verify the PDF 2.0 §11.4.5 / §11.6.6 wording against an ISO-purchased copy of ISO 32000-2:2020 before shipping a press-grade claim. The expected outcome is "identical exclusion list, identical §11.7.4.2 rule", but the actual citation should be against the purchased standard, not against the pdfa.org errata mirror.

---

## 7. Cross-references to existing code / tasks

- Task #48 (completed) wired the non-sep blend mode formulas to `tiny_skia` for the page renderer's CMYK output. Approach (B) means **the existing wiring is correct for the process lanes** and the new work for #46 is purely about the spot-lane behaviour.
- Task #51 (completed) pinned the BT.601 weights. Approach (B) keeps that pin.
- Task #46 (in-progress, the gating task) becomes: "extend the sidecar buffer so spot lanes ride alongside the process lanes through the SMask composite, and during composite the spot-lane blend function is per-§11.7.4.2 (separable+white-preserving → requested BM; else `Normal`)."
- Task #97 (OutputIntent CMYK ICC profile) gives us the device-link we need for press-accurate process-lane blends and is on the critical path before merging the spot-lane work.

---

## 8. Sources

Primary:
- `docs/spec/pdf.md` — local copy of ISO 32000-1:2008 (PDF 1.7). Specific line ranges cited above.
- ISO 32000-2:2020 PDF 2.0 — referenced via pdfa.org errata mirror at `https://pdf-issues.pdfa.org/32000-2-2020/clause08.html` and clause 11 errata; full normative text not redistributable.
- ISO 15930-7:2008 / ISO 15930-7:2010 (PDF/X-4) — scope summarised via `https://www.iso.org/standard/55843.html`, `https://www.iso.org/standard/42876.html`, and the prepressure.com PDF/X-4 explainer.

Secondary / cross-checks:
- W3C Compositing and Blending Level 1, `https://www.w3.org/TR/compositing-1/` — confirms the BT.601 weights and 3-vector restriction on non-sep formulas.
- PDF Reference 1.6 blend modes addendum, `https://printtechnologies.org/standards/files/pdf-reference-1.6-addendum-blend-modes.pdf` — Adobe's original formalisation of the four non-sep modes.
- A 20-years-of-PDF-transparency retrospective on pdfa.org, `https://pdfa.org/20-years-of-transparency-in-pdf/` — historical framing of the transparency model.
- A print-production explainer of PDF/X transparency handling at `https://callassoftware.com/blog-posts/understanding-transparency-in-prepress-pdf/` — corroborates that non-sep blends are rare in prepress PDFs and that practitioners are advised to avoid them; supports the "(B) is what real RIPs do" intuition without naming any specific RIP.

Documents inspected and found not to contain additional answers: the Autodesk-hosted Adobe transparency print-production whitepaper (binary fetch did not yield extractable text in this session).

`[unverified]` items flagged in-line:
- The exact wording of ISO 32000-2 §11.4.5 / §11.6.6 has not been verified against a purchased copy of the standard in this research pass; verification before shipping is item 5 in §6.

---

## 9. Bottom line for the design+impl brief

> When the page-group colour space is CMYK (with or without an OutputIntent ICC) and the document declares one or more spot inks, the renderer composites into a sidecar buffer of `(C, M, Y, K, spot_1, spot_2, …)`. The four channels `(C, M, Y, K)` are the spec's `DeviceCMYK` blending colour space and obey §11.3.5.3 fully, including the K-channel rule for `Hue/Saturation/Color/Luminosity`. The spot lanes are **not** a blend space; they ride beside it as §11.7.3 prescribes. Any non-separable or non-white-preserving blend mode runs on `(C, M, Y, K)` only and substitutes `Normal` on the spot lanes, per §11.7.4.2. There is no N-channel HSL math, and there is no invented luma weight for spot inks.

That is the architectural commitment issue #46 should make.
