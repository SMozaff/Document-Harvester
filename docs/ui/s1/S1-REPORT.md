# S-1 Locale Feasibility Report — 2026-11-28

**Branch/base:** `spike/s1-locale` from `b3dd0d222589ed399cd61d53697b1dc6a6f7708b`. **Configuration:** Slint and `slint-build` 1.19.0; product feature flags; `backend-winit` + software renderer; demo embeds `Vazirmatn-Regular.ttf` and sets `Vazirmatn` as default.

**Linux (Xvfb, 720×520):** Persian right-alignment, connected shaping and six tested Persian letter glyphs passed. Font metadata: Vazirmatn Regular 33.003; copyright The Vazirmatn Project Authors [1] [2]. Repository includes SIL OFL 1.1 text. One-window locale toggle passed: state token 7→9 remained 9 after switching. Long Persian text wrapped; controls stayed visible. **Mixed bidi failed:** an embedded Windows path reordered inside a Persian sentence; the separate LTR path stayed ordered.

**Packages:** Actual installer inspection **NOT RUN**. This sandbox is aarch64; its app build is ARM64 while bundled Linux PDFium, Debian manifest and AppImage workflow target x86-64. The AppImage, Debian and MSI file lists omit both the TTF and OFL text; bundle fallback is unproven.

**Windows:** GUI/font rendering, locale switching and screenshot **NOT RUN**; no Windows desktop or owner-provided S-1 UI run. Current-base Windows CI passed workspace tests/build and PDFium CLI smoke only, not GUI locale checks [3].

**Other verification:** formatting, strict Clippy, and `cargo build -p dh-ui` passed. `cargo test --workspace` failed at `dh-part1` (4 passed, 9 failed): the x86-64 PDFium library cannot load on this aarch64 host. The spike build and Linux reproduction script passed.

**Recommendation: STOP.** Do not freeze locale contracts until mixed-direction technical runs have a centrally tested, owner-approved strategy, the font and licence are included in target bundles, and Windows GUI checks pass.

Linux evidence: [English screenshot](/home/ubuntu/document-harvester/docs/ui/s1/linux-720x520-english.png), [Persian screenshot after state update](/home/ubuntu/document-harvester/docs/ui/s1/linux-720x520-persian-state-9.png). Reproduce with `bash spike/s1/check-linux.sh`.

## References
[1]: https://github.com/rastikerdar/vazirmatn "Vazirmatn upstream repository"
[2]: https://fonts.google.com/specimen/Vazirmatn "Google Fonts Vazirmatn family"
[3]: https://github.com/SMozaff/Document-Harvester/actions/runs/46103964944 "Windows Rust CI at the S-1 base commit"
