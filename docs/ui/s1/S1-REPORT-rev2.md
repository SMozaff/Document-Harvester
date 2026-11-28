# S-1 Locale Feasibility Report — rev2

**Branch/base:** `spike/s1-locale-rev2`, `5546e58439153842b4fe72af9b18e734daa5014d`. Prior S-1 commit `06b6711` is merged in PR #57. **Slint:** 1.19.0; product features; `backend-winit` + software renderer.

**Font:** Vazirmatn Regular 33.003 [3] [4]; repository SIL OFL 1.1. The `.slint` import embeds the TTF; the built binary rendered Persian with the source font moved aside [1]. Context7 did not index 1.19.0, so pinned-version behavior is established by the local test.

**Linux (Xvfb, 720×520):** Persian shaping, right alignment, long-text layout and controls passed. Baseline single-string path reordered. Rev2's prose/path/prose row passed in RTL order without bidi controls; its LTR path wrapped and remained fully visible. Filename/version/ID order also passed. Same-window state changed 7→9; English order passed.

**T8 proposal:** One `{path}` token parses as `Prose(prefix), LtrPath(value), Prose(suffix)`. Reject missing, duplicate or unknown placeholders and bidi controls. Render LTR as prefix/path/suffix and RTL as suffix/path/prefix. Layout docs describe horizontal placement but no RTL mirroring rule [2]. Architecture owner SMozaff approves; T8 implements after approval. Limit: one middle path tested; unit tests should cover valid, missing, duplicate, unknown and bidi-control inputs; Windows UI tests remain.

**Windows:** GUI/font/locale checks **NOT RUN**; this Linux sandbox has no Windows desktop. **Packaging:** no installer built or inspected on this aarch64 host; package files untouched. TTF embedding removes the loose-font need; proposed OFL entries are in `spike/s1/README.md`.

**Verification:** format, strict Clippy, `cargo build -p dh-ui`, locked spike build and Linux script passed. `cargo test --workspace`: 4 passed, 9 failed; x86-64 PDFium cannot load on aarch64.

**Recommendation: GO-WITH-ADAPTER**, subject to architecture-owner approval and Windows/package validation before freezing locale contracts.

Screenshots: [English](/home/ubuntu/document-harvester/docs/ui/s1/linux-rev2-720x520-english.png) · [Persian, segmented, state 9](/home/ubuntu/document-harvester/docs/ui/s1/linux-rev2-720x520-persian-segmented-state-9.png).

## References
[1]: https://docs.slint.dev/latest/docs/slint/guide/development/fonts/ "Slint custom font import and font handling"
[2]: https://docs.slint.dev/latest/docs/slint/reference/layouts/horizontallayout/ "Slint HorizontalLayout"
[3]: https://github.com/rastikerdar/vazirmatn "Vazirmatn upstream repository"
[4]: https://fonts.google.com/specimen/Vazirmatn "Google Fonts Vazirmatn family"
