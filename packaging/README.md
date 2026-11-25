# Release installer definitions

This directory contains only installer inputs and smoke scripts for D20. No
application logic lives here.

- `windows/nfpm-msi.yaml` installs `dh-app.exe` and the vendored `pdfium.dll`
  into one MSI `INSTALLFOLDER`, with Slint and PDFium license files.
- `linux/nfpm-deb.yaml` installs `dh-app` and the vendored `libpdfium.so` into
  `/usr/lib/document-harvester`, with license files under
  `/usr/share/doc/document-harvester`.
- `linux/build-appimage.sh` assembles the AppImage AppDir from the same inputs.
  The binary and `libpdfium.so` stay together in `usr/bin`, so the executable's
  own directory resolves the library with no system fallback.
- `linux/verify-deb.sh`, `linux/verify-appimage.sh` and
  `windows/verify-install.ps1` are the smoke checks: install/unpack into an
  empty location, assert the binary/library/license file set, then run
  `--pdfium-self-test` with networking disabled.

Each installer bundles the exact pinned vendored PDFium library from
`vendor/pdfium/<platform-x64>/` plus that release's `LICENSE` and `licenses/`
tree. The Slint Royalty-free 2.0 license text is kept in `licenses/`. Install
and runtime never download a library; `dh-app` loads PDFium from its own
executable directory.

The installer build tools and the AppImage runtime are recorded, with exact
versions, licenses and checksums, in [DEPENDENCIES.md](../docs/DEPENDENCIES.md).
The AppImage tool and runtime are downloaded only by the build script and their
SHA-256 values are checked before use; nothing is fetched during install or at
run time.

The smoke scripts use the repository `PDF_tests/` owner fixtures through the
path compiled into `--pdfium-self-test`. They do not fetch fixtures or any
other data during the smoke step. The built-in self-test is code-only here;
verification runs in CI.
