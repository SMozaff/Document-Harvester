# Dependencies and licensing

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

## Integrated provisional XLSX backend

| Component | Exact version | Role | License | Status |
|---|---:|---|---|---|
| `umya-spreadsheet` | `3.1.1` | XLSX create/read/update/write backend used by the project writer | MIT | Integrated provisionally in `dh-writer` and `dh-project` — evidence recorded, owner approval pending; deeper fidelity, lock, external-edit, recovery and undo validation remains open |
| `calamine` | `0.38.0` | Independent XLSX read-back in spike tests | MIT | Development-only dependency of `dh-xlsx-spike` and `dh-project` lifecycle tests; not a runtime application dependency |

The initial XLSX spike passed on Linux and Windows. See [MVP Phase 0 Technical Findings](MVP_PHASE0_FINDINGS.md) and [CI run 44912824313](https://github.com/SMozaff/Document-Harvester/actions/runs/44912824313). The backend is integrated provisionally, not fully approved for production release. Remaining gates include the complete transitive dependency/license audit, spreadsheet-application inspection, external-edit detection, lock behavior, safe undo, crash recovery and resource tests.

### Stage 5 task 5.1 audit — stopped on an unfixable runtime advisory

Base commit: `41bd926fda5c091febedbd412e9bc5bf38552f07`. **This is evidence only. The backend remains provisional; owner approval is pending. No replacement backend was investigated.**

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

## PDF backend candidate (Stage 2 task 2.1)

Registered for evaluation only. **No PDF backend is adopted**; no product crate may depend on any candidate. The isolated probe lives in `crates/dh-pdf-spike`.

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
| `pdf_oxide` | Digital PDF text/geometry and possible rendering | Registered in the [PDF backend candidate](#pdf-backend-candidate-stage-2-task-21) section above: exact version, license, resolved tree and API surface recorded; behavioural fixture evidence blocked | No |
| `unpdf` | Digital PDF candidate | Identity, API and license need verification | No |
| `pdfsink-rs` | Optional PDF fast path | Not investigated sufficiently | No |
| `rust_xlsxwriter` | New XLSX generation | Cannot alone modify an existing workbook for repeated append | No |
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
- The transitive licence audit for the exact resolved XLSX backend tree has been run and recorded in [Stage 5 task 5.1 audit](#stage-5-task-51-audit--stopped-on-an-unfixable-runtime-advisory); it is **blocked** by an unfixable `zip 8.6.0` advisory in the runtime tree. Continue to run `cargo deny` against the exact resolved tree before any production dependency approval, and do not treat the recorded audit as approval.
- Record platform system libraries, runtime requirements, redistribution conditions and clean-machine packaging results.
- Do not infer that a successful compile proves that a runtime dependency is bundled or present on a user's machine.
