# Project documentation

This directory contains the current MVP requirements, implementation plan, architecture, dependency evidence, owner decisions, scaffold inventory and acceptance records.

**Current implementation snapshot:** [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) — verified against main commit 2493f1fcbd4ee707aabd1d7a483537954fda11ef (24 November 2026). Read it before relying on dated status statements in older research or planning documents.

## Document map

- [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md) — current digital-only MVP product requirements and owner-confirmed workflow.
- [MVP Implementation Blueprint](MVP_IMPLEMENTATION_BLUEPRINT.md) — repository-aligned design, implementation phases and tests.
- [Team Implementation Plan](TEAM_IMPLEMENTATION_PLAN.md) — task register, current execution evidence and blockers.
- [Repository Status](REPOSITORY_STATUS.md) — concise verified implementation snapshot.
- [Acceptance Gates](ACCEPTANCE.md) — acceptance criteria and outstanding owner/platform/corpus gates.
- [Decision Status](DECISIONS.md) — resolved and pending owner decisions.
- [Dependencies and Licensing](DEPENDENCIES.md) — production dependency versions, licenses, platform requirements and historical candidate audits.
- [Architecture](ARCHITECTURE.md) — crate boundaries and integration responsibilities.
- [Scaffold Inventory](SCAFFOLD.md) — implemented interfaces and remaining stubs.
- [Corpus Plan](CORPUS.md) — independent reference-fixture requirements and evidence status.
- [Phase 0](PHASE0.md) and [MVP Phase 0 Findings](MVP_PHASE0_FINDINGS.md) — dated research and feasibility evidence; historical snapshots are labelled as such.
- [Stage S1 Owner Test](STAGE_S1_OWNER_TEST.md) — required manual project-lifecycle acceptance.

## Product boundaries

- Offline-first desktop application for Linux and Windows.
- Digital PDFs only in the current MVP. Image-only/scanned pages are reported as unsupported; OCR is not silently invoked.
- One workbook per project; approved captures are intended to append directly below prior captures.
- Private project/capture state belongs in OS application data, not beside or inside the workbook.
- Unassignable readable text must be surfaced with coordinates and a reason; no silent loss.

## Crate map

- dh-ir — shared intermediate representation.
- dh-core — geometry, grid reconstruction and core policies.
- dh-part1 — PDFium rendering and digital text extraction.
- dh-part2 — later OCR extension; outside the digital-only MVP.
- dh-part3 — page transforms and library-level region filtering.
- dh-writer — production XLSX creation/rebuild, verification, journal, history and undo primitives.
- dh-project — private project state, PDF copy-import and project-level capture operations.
- dh-ui / dh-app — Slint desktop shell and application integration.
- dh-xlsx-spike / dh-pdf-spike — isolated evidence only; not production dependency paths.

## Historical artifacts

The versioned files `document_harvester_manifest_v5.1.md`, `document_harvester_technical_manifest_v5.1.docx` and `pdf_extractor_dependencies_libraries_licences_v5.docx` are archived historical baselines/reports. They may contain superseded dependency or implementation statements and must not be used as the current status source. Use [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) and [DEPENDENCIES.md](DEPENDENCIES.md).

## Status discipline

Implemented library APIs are not equivalent to integrated UI workflows. CI, unit tests and merged PRs do not substitute for owner acceptance, real spreadsheet-app checks, independent corpus validation or clean-machine release verification. Keep dated research documents as historical evidence and update the canonical status snapshot and task register when code changes.
