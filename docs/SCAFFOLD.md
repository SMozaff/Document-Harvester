# Scaffold inventory

Handoff trace ID: `6afe930cccb6067c90112b89`. Recorded here for continuity; it is not a source of requirements.

Spec baseline: manifest v5.1, updated 2026-11-19. Every stub maps to a manifest section.

| Area | Stub | Manifest |
|---|---|---|
| IR types and extractor trait | `dh-ir::*` | Part 1 §5.2 (L276–L411) |
| Error type and result alias | `dh-ir::DhError`, `dh_ir::Result` | Part 1 §6.5 (L607–L617) |
| Column order, stitch mode, job config | `dh-core::config` | D1 (L102), D8 (L109), §5.12 (L533–L546) |
| Reason codes | `dh-core::reason` | §5.4 (L435–L451), Part 3 §4 (L1061–L1077) |
| Normalization | `dh-core::normalization` | §5.3 (L422–L434), D7 |
| Guard | `dh-core::guard` | §5.4 (L435–L451) |
| Numeric grammar/conversion | `dh-core::numeric` | §5.7 (L462–L480), D2/D13 |
| Grid and visual lines | `dh-core::grid` | §5.2, §5.9 (L485–L505), D12 |
| Continuation classifier | `dh-core::continuation` | §5.9, D8 |
| Triage and `triage.csv` | `dh-core::triage` | §6.2 (L591–L598) |
| State, `state_key`, recovery | `dh-core::state` | §5.12 (L529–L570) |
| Atomic publish and `.done` | `dh-core::publish` | §5.12 (L550–L556) |
| Writer and review worksheet | `dh-writer` | §5.11 (L521–L528), §6.3 (L599–L602) |
| Digital extractor | `dh-part1::extractor` | §7 (L618–L630) |
| Validated-font list | `dh-part1::fonts` | §5.2 (L414–L415) |
| Batch fast lane | `dh-part1::batch` | §5.13 (L571–L574) |
| OCR extractor/backends | `dh-part2` | Part 2 §3.1–3.3, §5.2 |
| Image/detection/merge/calibration/routing/lane | `dh-part2` | Part 2 §3.2, §3.4, §3.3, §3.6, §5.5, §5.6 |
| Region/containment/alerts/cache/export | `dh-part3` | Part 3 §3–§6 |
| UI skeleton | `dh-ui/ui/*.slint`, `dh-ui::UiBridge` | §6.4 (L604–L605), Part 3 §2, §4 |
| CLI entry point | `dh-app` | Part 1 §6, Part 2 §6, Part 3 §2 |

## What is real versus stubbed

Real: IR field set and trait signatures, `ColumnOrder::from_selection`, reason-code strings, cancellation token, lane-size formula, page cache, UI projection, CLI parsing.

Stubbed (panic with `todo!()`): every extractor, writer, normalization/guard/numeric/grid/continuation/triage operation, state/publish/recovery operations, region geometry and export.

## Filling a stub

1. Locate the manifest section in the table above and read it in full.
2. Check [DECISIONS.md](DECISIONS.md): if the behavior depends on a pending decision, leave it configurable and do not choose a default.
3. Check [PHASE0.md](PHASE0.md): if the stub needs a dependency, ground truth or corpus evidence, do not add it until the gate is closed.
4. Replace `todo!()` with the behavior, add a focused test, and update this inventory.
5. Keep the IR and the writer shared; do not add a second representation or a second output path.

## Verification

```
cargo check --workspace --all-targets
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```
