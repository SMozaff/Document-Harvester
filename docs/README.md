# Document Harvester scaffold

This repository currently contains a **specification, Rust scaffold, and an initial compiled desktop shell**; PDF extraction and project/workbook workflows are not yet implemented.

Source of truth: [`document_harvester_manifest_v5.1.md`](../document_harvester_manifest_v5.1.md). The documents in `docs/` summarize and navigate the manifest; they do not override it.

## MVP documents

- [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md) — plain-language requirements and confirmed workflow.
- [MVP Implementation Blueprint](MVP_IMPLEMENTATION_BLUEPRINT.md) — repository-aligned implementation phases, crate changes, append design and tests.

- [Team Implementation Plan](TEAM_IMPLEMENTATION_PLAN.md) — workstreams, task-by-task execution order, dependencies, deliverables, definitions of done and integration checkpoints.

## MVP Phase 0

- [MVP Phase 0 Technical Findings](MVP_PHASE0_FINDINGS.md) — preliminary PDF/XLSX candidate research, prototype plan and unresolved evidence gates.

## Layout

- `crates/dh-ir` — intermediate representation and `TableExtractor` trait.
- `crates/dh-core` — shared normalization, guard, numeric, grid, continuation, triage, state and publish policies.
- `crates/dh-writer` — shared `XlsxWriter` stub.
- `crates/dh-part1` — digital batch extraction.
- `crates/dh-part2` — OCR batch extension.
- `crates/dh-part3` — interactive region extraction.
- `crates/dh-ui` — compiled Slint MVP shell, About attribution panel and plain-data projection.
- `crates/dh-app` — desktop entry point (launches UI by default) and retained CLI mode.
- `crates/dh-xlsx-spike` — isolated Phase 0 XLSX feasibility tests; not a production dependency path.
- `data/` — reserved for versioned validated-font and normalization data.

## Commands

```
cargo check --workspace --all-targets
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo run -p dh-app -- --part 1 --column-order right_to_left
```

## Status

Slint 1.19.0 is integrated under its royalty-free desktop license with AboutSlint attribution; Linux and Windows CI compile checks pass. `umya-spreadsheet 3.1.1` remains a provisional workbook candidate, isolated in the spike crate. PDF extraction is not yet implemented, and full dependency/license and runtime acceptance gates remain open. See [DECISIONS.md](DECISIONS.md), [PHASE0.md](PHASE0.md) and [DEPENDENCIES.md](DEPENDENCIES.md).
