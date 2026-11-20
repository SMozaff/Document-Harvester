# Document Harvester scaffold

This repository currently contains a **specification and a Rust scaffold**, not a working extractor.

Source of truth: [`document_harvester_manifest_v5.1.md`](../document_harvester_manifest_v5.1.md). The documents in `docs/` summarize and navigate the manifest; they do not override it.

## MVP documents

- [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md) — plain-language requirements and confirmed workflow.
- [MVP Implementation Blueprint](MVP_IMPLEMENTATION_BLUEPRINT.md) — repository-aligned implementation phases, crate changes, append design and tests.

## MVP Phase 0

- [MVP Phase 0 Technical Findings](MVP_PHASE0_FINDINGS.md) — preliminary PDF/XLSX candidate research, prototype plan and unresolved evidence gates.

## Layout

- `crates/dh-ir` — intermediate representation and `TableExtractor` trait.
- `crates/dh-core` — shared normalization, guard, numeric, grid, continuation, triage, state and publish policies.
- `crates/dh-writer` — shared `XlsxWriter` stub.
- `crates/dh-part1` — digital batch extraction.
- `crates/dh-part2` — OCR batch extension.
- `crates/dh-part3` — interactive region extraction.
- `crates/dh-ui` — Slint UI skeleton and plain-data projection.
- `crates/dh-app` — CLI entry point stub.
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

Proposed decisions D7–D14 are not approved; no dependency, model or runtime is adopted; no bake-off or Definition of Done has passed. See [DECISIONS.md](DECISIONS.md), [PHASE0.md](PHASE0.md) and [DEPENDENCIES.md](DEPENDENCIES.md).
