# Dependencies and licensing

> **Current-path clarification (24 November 2026):** The integrated production tables above describe the current `main` dependency path. The audit sections below that describe the original `umya-spreadsheet` runtime tree and the isolated `rust_xlsxwriter` candidate evaluation are historical evidence from before PR #15; they must not be read as current production dependency claims. Current implementation status is in [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md).


The repository's own code is MIT licensed (`LICENSE`, copyright 2026 Smozaff). Third-party dependencies retain their own licenses; the repository license does not override them.

## Integrated desktop UI

| Component | Exact version | Role | License / distribution choice | Status |
|---|---:|---|---|---|
| `slint` | `1.19.0` | Desktop UI runtime; Winit backend and software renderer | Slint Royalty-Free Desktop, Mobile and Web Applications License 2.0; attribution is provided through the `AboutSlint` widget in the About panel | Integrated in `dh-ui`; Linux and Windows CI compile passes |
| `slint-build` | `1.19.0` | Compile `.slint` files in `dh-ui/build.rs` | Same Slint framework licensing choice | Integrated |
| `yeslogic-fontconfig-sys` | `6.0.1` (transitive) | Linux system font discovery | MIT | Linux build requires `libfontconfig1-dev`; runtime requires Fontconfig |

Slint feature selection in `crates/dh-ui/Cargo.toml`: `std`, `backend-winit`, `renderer-software`, `compat-1-18`, and `accessibility`; default features are disabled. The desktop app uses a general-purpose desktop target, not an embedded system.

**Slint licensing condition:** the Royalty-Free License permits desktop applications at no royalty but requires either the `AboutSlint` widget in an About screen/dialog accessible from the top-level UI or the Slint attribution badge on a public application webpage. The app implements the About panel route. Revisit this license if the product is later embedded, exposes Slint APIs as an SDK, or the About attribution is removed.

References:
- [Slint 1.19.0 crate and feature metadata](https://docs.rs/crate/slint/1.19.0)
- [Slint license choices](https://github.com/slint-ui/slint/blob/master/LICENSE.md)
- [Royalty-Free License 2.0](https://slint.dev/agreements/slint-royalty-free-license.pdf)
- [AboutSlint widget](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/misc/aboutslint/)

## Integrated XLSX backend — rebuild from capture data (Stage 5 writer core)

| Component | Exact version | Role | License | Status |
|---|---:|---|---|---|
| `rust_xlsxwriter` | `0.104.0` | XLSX create/write backend used by the production `dh-writer` rebuild path | `MIT OR Apache-2.0` | Adopted in the Stage 5 writer core; rebuild writes the full workbook from the ordered capture list to a temporary file, reopens it with the independent `calamine` reader, verifies every value and type and atomically replaces the target. Production adoption is implemented on `main`; Stage 5 owner/platform acceptance remains pending |
| `calamine` | `0.38.0` | Independent XLSX read-back and verification reader for the production rebuild path and lifecycle tests | `MIT` | Runtime verification dependency of `dh-writer`; development/evidence reader for `dh-xlsx-spike` and `dh-project` lifecycle tests |
| `sha2` | `0.11.1` | SHA-256 hashing of committed workbooks for the external-change check | `MIT OR Apache-2.0` | Adopted in `dh-writer` for committed-workbook integrity comparison |

The provisional `umya-spreadsheet = 3.1.1` backend is **no longer integrated** in the production writer (`dh-writer`); it remains only as the evidence backend inside the isolated `dh-xlsx-spike` crate. The unfixable `zip 8.6.0` advisory set therefore no longer reaches the release `dh-app` runtime tree.

The initial XLSX spike passed on Linux and Windows. See [MVP Phase 0 Technical Findings](MVP_PHASE0_FINDINGS.md) and [CI run 44912824313](https://github.com/SMozaff/Document-Harvester/actions/runs/44912824313). The Stage 5 writer core supersedes the provisional integration for the rebuild path described above; the candidate audit and findings in the next section are the evidence for that adoption. Remaining gates include real spreadsheet-application inspection, OS-level lock verification, owner/platform acceptance and performance/resource thresholds.

### Historical Stage 5 task 5.1 audit — original umya runtime path (superseded)

Base commit: `41bd926fda5c091febedbd412e9bc5bf38552f07`. **Historical state at that base commit:** evidence only; the backend remained provisional and no replacement had yet been adopted.

**Method and exact commands (22 November 2026, Linux x86_64):**

- Tool: `cargo-deny 0.22.0`, installed with `cargo install cargo-deny --locked`. The RustSec advisory database was fetched successfully by cargo-deny; no offline fallback was needed.
- Audit-only config at `/tmp/opencode/deny.toml` (not committed). It deliberately used an empty allow-list so nothing was suppressed:

```toml
[licenses]
allow = []
confidence-threshold = 0.8

[advisories]
version = 2
yanked = "warn"
ignore = []
```

- Runtime set (what ships in a release build of `dh-app`): `cargo deny --config /tmp/opencode/deny.toml --manifest-path crates/dh-app/Cargo.toml --exclude-dev --locked check licenses advisories`
- Full workspace set including dev dependencies: `cargo deny --config /tmp/opencode/deny.toml --locked check licenses advisories`
- Package sets: `cargo tree -p dh-app -e normal --prefix none --format "{p}"`, `cargo tree -p dh-xlsx-spike -e all --prefix none --format "{p}"`, `cargo tree -p dh-project -e all --prefix none --format "{p}"` and `cargo metadata --format-version 1` for the license fields.
- Both cargo-deny runs exited 5 with `advisories FAILED, licenses FAILED`. The license failure is an artefact of the empty allow-list (every licence, including plain MIT, is reported as "not explicitly allowed"); it is not a rejected-license finding. The advisory failure is real.

**Tree counts (unique name/version pairs, host target):**

| Set | Packages | Notes |
|---|---:|---|
| Release `dh-app` (`cargo tree -p dh-app -e normal`) | 391 | Includes `umya-spreadsheet 3.1.1`, `zip 8.6.0`, `flate2 1.1.11`, `quick-xml 0.41.0`, `roxmltree 0.21.2`; no `calamine` |
| Dev-only (workspace dev edges minus the release set) | 13 | `calamine 0.38.0`, `zip 9.8.1`, `quick-xml 0.42.0` and 10 of their transitive packages |
| Isolated spike normal tree (evidence only, never shipped) | 80 | `dh-xlsx-spike` plus 79 transitive packages |

**Missing, unrecognized, custom and copyleft licences.** No package in either set has an empty `license` field. No package has a mandatory GPL/LGPL/AGPL/MPL licence. The following declared expressions were flagged and are recorded verbatim:

| Package(s) | Version | Declared licence text | Where |
|---|---|---|---|
| `slint`, `slint-macros`, `i-slint-common`, `i-slint-core`, `i-slint-core-macros`, `i-slint-compiler`, `i-slint-renderer-software`, `i-slint-backend-selector`, `i-slint-backend-winit` | `1.19.0` | `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` | Runtime; `LicenseRef-*` is a custom SPDX identifier. The GPL branch is an alternative, not mandatory; the documented Royalty-Free 2.0 choice with `AboutSlint` attribution is unchanged |
| `linux-raw-sys` | `0.4.15`, `0.12.1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | Runtime; `WITH LLVM-exception` is not recognised by the empty allow-list but is a valid SPDX exception |
| `rustix` | `0.38.44`, `1.1.6` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | Runtime; same as above |

The 13 dev-only packages are `MIT`, `Apache-2.0`, `BSD-3-Clause`, `Unicode-3.0`, `Zlib`, `0BSD`, `ISC`, `Unlicense` or combinations of those; none is missing, custom or copyleft. `umya-spreadsheet 3.1.1` and `calamine 0.38.0` are both `MIT`.

**System libraries and runtime requirements observed in the tree:** `yeslogic-fontconfig-sys 6.0.1` needs Fontconfig headers at build time and Fontconfig at Linux runtime; `wayland-sys 0.31.11` is a Wayland client binding; `linux-raw-sys` contains raw Linux syscall bindings but no external C library. Neither `umya-spreadsheet`, `calamine`, `zip` nor `flate2` adds a `-sys` crate or C library. This is a tree observation, not a clean-machine packaging result: a successful compile does not prove that any of these libraries is bundled or present on a user's machine.

**Advisories — STOP.** The exact resolved runtime tree contains `zip 8.6.0`, reached through `umya-spreadsheet 3.1.1 → dh-writer → dh-app`. cargo-deny reports eight advisories for it:

| Advisory | Patched in | Advisory | Patched in |
|---|---|---|---|
| `RUSTSEC-2026-0367` | `>= 9.0.0` | `RUSTSEC-2026-0429` | `>= 9.0.0` |
| `RUSTSEC-2026-0368` | `>= 9.0.0` | `RUSTSEC-2026-0545` | `>= 9.0.0` |
| `RUSTSEC-2026-0369` | `>= 9.0.0` | `RUSTSEC-2026-0546` | `>= 9.0.0` |
| `RUSTSEC-2026-0547` | `>= 9.0.0` | `RUSTSEC-2026-0548` | `>= 9.0.5` |

cargo-deny states: "1 dependent has version requirements that preclude updating: `umya-spreadsheet = 3.1.1` — `zip = '^8.6.0'`". No patched 8.x release exists, so the advisory cannot be resolved by a lockfile update. This is an unfixable advisory in the runtime tree: task 5.1 is stopped, fidelity and resource work were not started, and no replacement was evaluated. Recommendation only (no change made): a repository `deny.toml` and a cargo-deny CI check are needed before any future dependency approval; that is a separate owner decision.

### Historical Stage 5 task 5.1 — `rust_xlsxwriter 0.104.0` candidate audit (superseded by production adoption)

Base commit: `8cfee495eee8329741d0099024f8f1a3dec1d035` (current `origin/main`). **Historical decision at audit time:** evidence only; no backend had yet been adopted and no production dependency had changed. `dh-writer` then depended on `umya-spreadsheet =3.1.1`; the candidate was added only to the isolated `dh-xlsx-spike` crate, and the candidate runtime tree was measured by temporarily substituting the `dh-writer` manifest entry locally and restoring it immediately afterwards. The committed `Cargo.lock` gains 17 package entries and changes no existing package version.

**Stage 5 writer core update (23 November 2026, same stage):** the approved rebuild-from-capture-data design was implemented in the writer core: `dh-writer` now depends on `rust_xlsxwriter =0.104.0` directly, writes the full workbook from the ordered capture list to a temporary file, reopens it with the independent `calamine` reader, verifies every value and type, and atomically replaces the target; the external-change check compares the on-disk workbook SHA-256 with the hash of the last commit; the failed-write path keeps the last valid workbook. The provisional `umya-spreadsheet` dependency was removed from the production writer and remains evidence-only inside `dh-xlsx-spike`. Owner acceptance of the design and of the captured-feature-loss consequences remains pending.

**Stage 5 recovery/history/undo update (24 November 2026, same stage):** `dh-writer` gained a direct `serde_json = "=1.0.154"` dependency for the JSON-lines transaction journal and a dev-only `zip = "=9.8.1"` dependency used as the raw-OOXML oracle in the writer tests. Both packages already appear in the audited dependency plan above (`serde_json` is the existing private-state serializer in `dh-project`; `zip 9.8.1` is the recorded dev-only archive reader). No new third-party package or version was introduced.

**Candidate identity and configuration (package manifest, 23 November 2026):**

| Fact | Value |
|---|---|
| Crate / version | `rust_xlsxwriter 0.104.0` (crates.io, exact `=0.104.0` in the spike) |
| Declared licence | `MIT OR Apache-2.0` |
| Declared MSRV | `1.88.0`; local toolchain `rustc 1.100.0` |
| Required dependency | `zip = "9.0"` with feature `deflate`, `default-features = false`; resolves to `zip 9.8.1` |
| Optional dependencies | All off by default (`default = []`): `chrono`, `jiff`, `polars`, `rust_decimal`, `ryu`, `serde`, `zmij`, `wasm`, `constant_memory`, `enhanced_autofit`, `zlib`, `test-resave` |
| Applicability | Write-only API. It cannot edit an existing workbook in place; the production application now rebuilds the workbook from its ordered private capture list through the implemented writer path. See [DECISIONS.md](DECISIONS.md) and [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md). |

**Method and exact commands (Linux x86_64, 23 November 2026):** the same temporary, uncommitted `/tmp/opencode/deny.toml` with `allow = []`, `confidence-threshold = 0.8`, `advisories.version = 2`, `yanked = "warn"`, `ignore = []`; tool `cargo-deny 0.22.0`; advisory database fetched successfully.

- Candidate release set (temporary `dh-writer` substitution): `cargo tree -p dh-app -e normal --prefix none --format "{p}"` then `cargo deny --config /tmp/opencode/deny.toml --manifest-path crates/dh-app/Cargo.toml --exclude-dev --locked check licenses advisories`
- Full workspace set (candidate in the spike, temporary substitution in `dh-writer`): `cargo tree --workspace -e all --prefix none --format "{p}"`, `cargo tree -p dh-xlsx-spike -e all --prefix none --format "{p}"`, `cargo metadata --format-version 1`, then `cargo deny --config /tmp/opencode/deny.toml --locked check licenses advisories`
- Reachability/feature check: `cargo tree -p dh-app -e features -i zip@8.6.0 --prefix none`
- The `dh-writer` manifest was restored before any test, commit or CI run. No `deny.toml` was added to the repository and no cargo-deny CI job was created.

**Results:**

| Check | Candidate runtime tree | Full workspace tree |
|---|---|---|
| `advisories` | **ok**, zero advisories or yanked warnings | **FAILED**, solely the eight known `zip 8.6.0` advisories inherited from the baseline umya spike |
| `licenses` | FAILED by construction: the empty allow-list rejects every licence, including MIT. No missing field, no mandatory copyleft | Same artefact, plus the licence texts listed below |

Tree counts (unique name/version pairs, host target):

| Set | Packages | Notes |
|---|---:|---|
| Hypothetical release `dh-app` with the candidate in `dh-writer` | 352 | Includes `rust_xlsxwriter 0.104.0` and `zip 9.8.1`; no `umya-spreadsheet`, no `zip 8.6.0`, no `calamine`. Thirty-nine fewer packages than the current umya release tree (391) |
| Workspace packages outside that release set | 126 | Dev/test/build edges plus spike-only normal edges. Still contains `umya-spreadsheet 3.1.1`/`zip 8.6.0` because the baseline spike tests remain, plus `calamine 0.38.0`, `zip 9.8.1` and `slint-build 1.19.0` |
| Isolated spike, all edge kinds | 383 | Both candidates present for comparison; never shipped |
| Historical committed release tree (pre-PR #15) | 391 | `dh-app → dh-writer → umya-spreadsheet 3.1.1 → zip 8.6.0`; superseded by the production writer adoption in PR #15 |

Candidate runtime licence expressions: 124 single permissive, 215 multi-permissive, 9 Slint crates with `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` (alternative, not mandatory; Royalty-Free 2.0 with `AboutSlint` remains the documented choice), and 4 declarations of `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` (`linux-raw-sys 0.4.15`/`0.12.1`, `rustix 0.38.44`/`1.1.6`). No licence field is empty; no mandatory GPL/LGPL/AGPL/MPL package exists. The packages outside the release set add `slint-build 1.19.0` (same Slint expression) and `libbz2-rs-sys 0.2.7` with `bzip2-1.0.6` (cargo-deny recognises it as "bzip2 and libbzip2 License v1.0.6" but has no additional metadata; it is not copyleft). The new transitive packages introduced by the candidate are `bzip2`, `deflate64`, `deranged`, `libbz2-rs-sys`, `lzma-rust2`, `num-conv`, `pbkdf2`, `powerfmt`, `ppmd-rust`, `sha1`, `time`, `time-core`, `zstd`, `zstd-safe` and `zstd-sys`, all from `zip 9.8.1`'s default feature set; their declared licences are permissive. This is manifest metadata, not a legal opinion.

**`zip 8.6.0` advisory reachability in the current umya path (analysis, not a fix):** `umya-spreadsheet 3.1.1` declares only `zip 8.6.0` with `features = ["deflate"]`, `default-features = false`; `cargo tree -e features` confirms only `deflate` and its flate2/zlib-rs internals are enabled. Source inspection of the crate at version 3.1.1 finds `ZipArchive::new` and 70 `by_name(...)` reads in the reader, and `ZipWriter::new`/`start_file` in the writer; it finds no `extract`, `ZipStreamReader`, symlink handling, `by_name_decrypt`, `aes-crypto` or `legacy-zip` use. Therefore:

| Advisories | Required trigger | Reachable through current umya code? |
|---|---|---|
| `RUSTSEC-2026-0367`, `-0368`, `-0369`, `-0429`, `-0546` | `ZipArchive::extract` / stream extraction (SUID bits, symlinks, permissions, allocation, absolute paths) | No — no extraction API is called |
| `RUSTSEC-2026-0548` | `ZipStreamReader` local-header name mismatch | No — no `ZipStreamReader` is used |
| `RUSTSEC-2026-0545` | AES entry read with the `aes-crypto` feature and `by_name_decrypt` | No — feature disabled, method unused |
| `RUSTSEC-2026-0547` | Legacy Shrink/Reduce/Implode decoders with the `legacy-zip` feature | No — feature disabled |

This analysis explains why cargo-deny's version-based advisory failure has no demonstrated exploit path in today's runtime code. It does **not** patch the version, remove the cargo-deny finding, approve the backend, or change task 5.1's status. Any future use of extraction, AES or legacy ZIP features would invalidate it. A repository `deny.toml` and cargo-deny CI check remain a separate owner decision.

## Bundled PDFium renderer and digital-text adapter (Stage 2 tasks 2.3–2.4; Stage 3 tasks 3.1–3.2)

**Owner decision recorded in this task.** PDFium is the viewer renderer and is bundled with the application; install and run time are offline and download nothing. Imported PDFs are copied into the project's app-data folder and the copy path is recorded in project state. The viewer is implemented before writer/append work. No beta dependencies and no OCR. This is an explicit owner decision, not a silent change to D5: the manifest's "single executable" wording is the constraint the owner has resolved for the viewing dependency by approving a co-located native library. PDFium is the **only** PDF backend in the production path: Stage 3 reuses the same adopted binding and native library for digital text, so no second engine, no new dependency and no OCR were introduced. Digital extraction never uses rendered pixels (manifest Part 3 §6). The separate `pdf_oxide` extraction candidate below remains registered for evaluation only and is unchanged.

| Component | Exact version | Role | License | Status |
|---|---:|---|---|---|
| `pdfium-render` | `0.9.6` (`=0.9.6`, crates.io) | Rust binding used by the `dh-part1` renderer and digital-text adapter | `MIT OR Apache-2.0` | Adopted by owner decision for the viewer renderer and reused unchanged for Stage 3 digital text; latest stable crates.io release at the time of recording (published 2026-11-21) |
| PDFium prebuilt library | `chromium/8125` (bblanchon/pdfium-binaries, published 2026-10-26) | Native renderer shipped as `libpdfium.so` / `pdfium.dll` | PDFium `BSD-3-Clause`; release packaging `MIT`; upstream third-party notices retained | Adopted by owner decision; pinned, vendored and loaded from the app directory |

**Binding identity and platform evidence (crates.io registry metadata, 23 November 2026):**

- `pdfium-render 0.9.6`: license `MIT OR Apache-2.0`, declared MSRV 1.61, edition 2021, stable (not yanked; `max_stable_version = 0.9.6`), repository <https://github.com/ajrcarey/pdfium-render>. Exact pin `=0.9.6` in `crates/dh-part1/Cargo.toml`.
- Features enabled: `pdfium_8125` and `thread_safe`; `default-features = false` so the `image_*`, `bindings`/`bindgen`, WASM and `static` features are off. The normal non-WASM dependency set is `bitflags`, `bytemuck`, `bytes`, `chrono`, `itertools`, `libloading`, `log`, `maybe-owned`, `once_cell`, `utf16string` and `vecmath`. `thread_safe` is required because `dh-app` calls the renderer off the UI thread.
- `pdfium_8125` is the highest FPDFAPI revision exposed by this binding. The newer `chromium/8165` binary release has no matching stable binding feature, so `chromium/8125` is the newest binary release with an exact binding match. Compatibility is asserted by the pinned pair, not by an ABI assumption.

**Binary identity, bundling and platform evidence:**

- Release assets, checksums and the exact download URLs are recorded in [vendor/pdfium/README.md](../vendor/pdfium/README.md). Both `pdfium-linux-x64.tgz` and `pdfium-win-x64.tgz` are published for `chromium/8125`; the vendored files are `libpdfium.so` (8,049,768 bytes) and `pdfium.dll` (7,586,816 bytes).
- Method: the two release archives were downloaded once during development, unpacked and committed under `vendor/pdfium/linux-x64/` and `vendor/pdfium/win-x64/` together with their `LICENSE`, `VERSION` and `licenses/` notices. Nothing is fetched by a build script, installer or at run time; the only network access in a normal build is Cargo fetching registry crates.
- `crates/dh-app/build.rs` copies the host-platform library into the Cargo output directory next to the executable. The loader uses `std::env::current_exe()`'s parent directory plus `Pdfium::pdfium_platform_library_name()`; there is no system-library fallback. CI runs the built release app with `--pdfium-self-test` on both Ubuntu and Windows to prove the co-located library loads and renders.
- Platform evidence: Linux x64 compiles, loads and renders all three owner PDF fixtures in the local `dh-part1` tests; the built release app loads the co-located library and renders all three owner fixtures in `--pdfium-self-test`. Both platforms passed in [run 45136334739](https://github.com/SMozaff/Document-Harvester/actions/runs/45136334739) (code head `3d01539`); task-level links are in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md).
- Redistribution: each vendored directory retains the upstream `LICENSE` and full `licenses/` set, including PDFium's BSD-3-Clause notice and the bundled third-party notices. These are binary redistribution notices, not a legal opinion.

**Status:** adopted by the owner for the viewer renderer. Stage 3 uses the same pinned binding and library for digital text; this is reuse of an approved dependency, not a second adoption or a new capability licence review. No audit beyond this task's identity/license/platform record is claimed here. Text extraction uses PDFium's text layer and character geometry, never rendered pixels. The source-PDF copy-vs-reference policy recorded as pending in blueprint §4.4 is resolved by the owner's copy-import decision; content hashing for extraction cache invalidation remains a later-stage concern and is not implemented here.

## PDF backend candidate (Stage 2 task 2.1)

Registered for evaluation only. **No additional extraction backend is adopted**; no product crate may depend on any candidate listed here. The adopted PDFium binding above is now also used for Stage 3 digital text, so this probe remains isolated in `crates/dh-pdf-spike` and is not part of the product path.

| Component | Exact version | Role | License | Status |
|---|---|---|---|---|
| `pdf_oxide` | `0.3.78` (`=0.3.78`, crates.io) | Candidate digital-PDF source: page metadata and boxes, character/span geometry, font metadata, page rendering | `MIT OR Apache-2.0` (declared in the crate manifest; `LICENSE-MIT` and `LICENSE-APACHE` ship in the package) | Isolated probe only; not a production dependency |

- Identity and source: crates.io `pdf_oxide` 0.3.78; upstream <https://github.com/yfedoseev/pdf_oxide>; documentation <https://pdf.oxide.fyi>. Minimum Rust version declared by the crate: 1.88; the repository toolchain is stable (Rust 1.100.0 at the time of probing).
- Features probed: crate defaults (`icc`, `legacy-crypto`) plus `rendering`, which pulls `tiny-skia`, `fast_image_resize`, `fontdb`, `harfrust`, `hayro-jbig2` and `hayro-jpeg2000`. The `ocr`, `ml`, `gpu`, `table-ml`, `python`, `parallel`, `signatures` and `fips` features are **not** enabled; OCR is permanently out of scope.
- Resolved tree: 161 packages including the probe crate, recorded from `cargo tree -p dh-pdf-spike`. Adding the probe to the workspace lockfile added 30 packages and changed **no existing package version**. Declared licenses across that tree are all permissive (`MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `Zlib`, `0BSD`, `Unlicense`, `CC0-1.0`, `Unicode-3.0`); no GPL, LGPL or AGPL package is declared. Where the workspace already used an older minor of the same crate (`harfrust`, `skrifa`, `read-fonts`, `write-fonts`, `nom`, `roxmltree`, `weezl`, `jiff`, `taffy`), the lockfile now carries both versions rather than upgrading the existing one. This is registry metadata, not a legal audit: `cargo deny` against the exact resolved tree is still required before any adoption.
- Packaging route: no `-sys` crate and no C system library appears in the probed tree. `fontdb` and `fontconfig-parser` are pure Rust, so this candidate does **not** add Fontconfig linkage of its own (contrast `slint`'s `yeslogic-fontconfig-sys`). That is consistent with the single-executable constraint (D5), but it is a dependency-tree observation, not a measured clean-machine packaging result.
- Platform evidence: compiles, passes `cargo check`, `cargo clippy` and `cargo test -p dh-pdf-spike` on Linux. **Windows is not yet verified** for this crate; no CI run exists for the probing branch yet.
- Reachable API surface, confirmed by compiling the probe: `Pdf::open`, `Pdf::from_bytes`, `page_count`, `page_rotation`, `page_media_box`, `page_crop_box`, `extract_chars`, `extract_spans`, `render_page`.
- API friction found: `RenderOptions` contains a private field (`scale_override`), so struct-update syntax from outside the crate fails to compile (`E0451`); the supported route is `RenderOptions::with_dpi(dpi).as_raw()`.
- Explicit-failure behaviour: malformed bytes and a missing path both return `Err` without panicking or reporting an empty document (`malformed_input_is_reported_as_an_error`, `missing_file_is_reported_as_an_error`).
- **Behavioural evidence is missing and this gate is blocked.** Character/span geometry, font metadata, reading order, page rotation, MediaBox/CropBox values, rendering output, Persian/English/mixed-script handling, and encrypted/corrupt/image-only behaviour have **not** been checked against fixtures. Owner decision A (permitted fixture PDFs and the annotator of expected values) is unanswered, and generating synthetic PDFs as ground truth is prohibited. Windows evidence also needs a CI run.
- Adoption decision: **none recorded.** Owner decision B (conditional approval to adopt a backend if the spike evidence passes) is unanswered, so there is no decision record and no product crate may use this backend.

## Unselected candidates

### MVP PDF / XLSX alternatives

| Component | Role | License / note | Adopted |
|---|---|---|---|
| `pdf_oxide` | Digital PDF text/geometry and possible rendering | Registered in the [PDF backend candidate](#pdf-backend-candidate-stage-2-task-21) section above: exact version, license, resolved tree and API surface recorded; behavioural fixture evidence blocked. Stage 3 uses the already-adopted PDFium instead. | No |
| `unpdf` | Digital PDF candidate | Identity, API and license need verification | No |
| `pdfsink-rs` | Optional PDF fast path | Not investigated sufficiently | No |
| `rust_xlsxwriter` | New XLSX generation | `0.104.0` audited in isolation: MIT OR Apache-2.0, clean runtime advisories, write-only. The isolated candidate audit predates production adoption; the implemented rebuild path is described in the writer-core update above and [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) | Yes — production rebuild path; Stage 5 acceptance pending |
| `pdf-xlsx` | Not primary | AGPL-3.0 / commercial; not selected | No |

### Part 2 OCR candidates (outside digital-only MVP)

| Component | Role | Packaging route | License | Adopted |
|---|---|---|---|---|
| Rasterizer (candidate `rasterrocket`) | Page rendering | To be recorded | To be recorded | No |
| Raster table detection / layout model | Table and cell structure | To be recorded | To be recorded | No |
| Tesseract + `fas`/`eng` traineddata | OCR candidate | Native runtime | To be recorded | No |
| PaddleOCR + models | OCR candidate | ONNX/native/excluded (D10) | To be recorded | No |
| xberg | Optional VLM | To be recorded | To be recorded | No |
| Local VLM engine | Offline inference | To be recorded | To be recorded | No |

## Dependency audit policy

- Exact direct versions are pinned in Cargo manifests; the resolved lockfile and dependency tree are generated by CI and retained as workflow artifacts.
- The eight-advisory `zip 8.6.0` block belongs to the historical `umya-spreadsheet` candidate tree, not the production `rust_xlsxwriter` path. The candidate audit and production adoption evidence are retained above. Re-run `cargo deny` against the exact release dependency tree as part of release verification; do not infer release approval from the historical candidate audit.
- Record platform system libraries, runtime requirements, redistribution conditions and clean-machine packaging results.
- Do not infer that a successful compile proves that a runtime dependency is bundled or present on a user's machine.
