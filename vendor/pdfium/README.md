# Bundled PDFium binaries

These files are the exact prebuilt PDFium libraries used by the offline viewer renderer. They are committed so that installing and running the application never downloads anything.

| Item | Value |
|---|---|
| Upstream project | <https://github.com/bblanchon/pdfium-binaries> |
| Release tag | `chromium/8125` (published 2026-10-26) |
| PDFium version file | `MAJOR=158`, `MINOR=0`, `BUILD=8125`, `PATCH=0` (both `VERSION` files) |
| Binding compatibility | `pdfium-render 0.9.6` feature `pdfium_8125` (its highest supported FPDFAPI revision) |

| Directory | Release asset | SHA-256 of the downloaded `.tgz` | Installed file |
|---|---|---|---|
| `linux-x64/` | `pdfium-linux-x64.tgz` | `e1c3ce6e270cb5e0c20816b379ad9f2c1309972697c505f508de92bf5e7d6659` | `lib/libpdfium.so` → `libpdfium.so` |
| `win-x64/` | `pdfium-win-x64.tgz` | `9a964261760cff52570aabf038ab346d3a5034dd8b30d0d5fe8861235d356b8b` | `bin/pdfium.dll` → `pdfium.dll` |

Download URLs:

- <https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/8125/pdfium-linux-x64.tgz>
- <https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/8125/pdfium-win-x64.tgz>

Each directory keeps the upstream `LICENSE` (MIT, copyright Benoit Blanchon for the packaging) and the upstream `licenses/` notices, including `licenses/pdfium.txt` (BSD-3-Clause, "Copyright 2014 The PDFium Authors"). Redistribution keeps those notices with the binary. Only the native library and the notices are vendored; headers and CMake files from the release archives are not needed by the Rust binding.

`crates/dh-app/build.rs` copies the platform library next to the built executable. At run time the renderer resolves the library through `std::env::current_exe()`'s parent directory and the platform library filename; it never falls back to a system library and never downloads one.
