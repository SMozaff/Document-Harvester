# Provisional extraction baseline — NOT ground truth

These `.baseline` files are deterministic serialisations of **PDFium's own**
digital-text extraction and Stage 3 geometry reconstruction for the owner PDFs
in `PDF_tests/`:

- `01.baseline` — `PDF_tests/01.pdf`
- `02.baseline` — `PDF_tests/02.pdf`
- `03.baseline` — `PDF_tests/03.pdf`

They include page boxes, positioned runs, run order, exposed font metadata,
unassigned text, warnings and reconstructed cells. They exist only as
**regression baselines**: they detect changes in our adapter or reconstruction
logic.

They are **not independent ground truth**. They were produced by the same
bundled PDFium backend used by the extractor, and no human or independent
reference process annotated the expected text, cell boundaries, reading order
or values. Stage 3 task 3.7 (independent fixture corpus) and task 3.8
(corpus validation) therefore remain open, and this baseline cannot be used to
mark Stage 3 accepted.

The baseline can be regenerated deliberately with:

```sh
DH_UPDATE_EXTRACTION_BASELINE=1 cargo test -p dh-part1 --test extraction
```

That variable is never set in CI. The ordinary test always compares against
the checked-in file. The comparison normalises CRLF to LF and the files are
pinned to LF with `.gitattributes`.
