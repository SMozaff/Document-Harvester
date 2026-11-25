#!/usr/bin/env bash
# Assemble the AppImage for Document Harvester (D20).
#
# Inputs are the release build outputs and the AppDir sources committed under
# packaging/linux/appimage. Nothing is downloaded at install or run time; the
# pinned appimagetool is downloaded here, at build time only, and its SHA-256
# is verified before use. See docs/DEPENDENCIES.md for the tool record.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
appdir="${repo_root}/target/appimage/AppDir"
output="${1:-${repo_root}/target/document-harvester.AppImage}"
tool="${repo_root}/target/appimage/appimagetool-x86_64.AppImage"
tool_sha256='b90f4a8b18967545fda78a445b27680a1642f1ef9488ced28b65398f2be7add2'
tool_url='https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage'

# Version must be set by the release workflow; no implicit default.
: "${PACKAGE_VERSION:?PACKAGE_VERSION must be set}"

mkdir -p "$(dirname "${appdir}")" "$(dirname "${tool}")" "$(dirname "${output}")"
rm -rf "${appdir}"
mkdir -p "${appdir}/usr/bin" "${appdir}/usr/share/doc/document-harvester/pdfium-licenses"

# Binary and vendored library stay together in usr/bin: the loader resolves
# PDFium from the executable's own directory with no system fallback.
install -m 0755 "${repo_root}/target/release/dh-app" "${appdir}/usr/bin/dh-app"
install -m 0755 "${repo_root}/target/release/libpdfium.so" "${appdir}/usr/bin/libpdfium.so"

install -m 0755 "${repo_root}/packaging/linux/appimage/AppRun" "${appdir}/AppRun"
install -m 0644 "${repo_root}/packaging/linux/appimage/document-harvester.desktop" \
  "${appdir}/document-harvester.desktop"
install -m 0644 "${repo_root}/packaging/linux/appimage/document-harvester.xpm" \
  "${appdir}/document-harvester.xpm"

install -m 0644 "${repo_root}/packaging/licenses/SLINT-ROYALTY-FREE-2.0.md" \
  "${appdir}/usr/share/doc/document-harvester/SLINT-LICENSE.md"
install -m 0644 "${repo_root}/vendor/pdfium/linux-x64/LICENSE" \
  "${appdir}/usr/share/doc/document-harvester/PDFIUM-LICENSE.txt"
cp -R "${repo_root}/vendor/pdfium/linux-x64/licenses/." \
  "${appdir}/usr/share/doc/document-harvester/pdfium-licenses/"

# Build-time tool download only; checksum is the pin recorded in DEPENDENCIES.md.
if [[ ! -f "${tool}" ]]; then
  curl -fsSL -o "${tool}" "${tool_url}"
fi
actual="$(sha256sum "${tool}" | awk '{print $1}')"
if [[ "${actual}" != "${tool_sha256}" ]]; then
  echo "appimagetool checksum mismatch: expected ${tool_sha256}, got ${actual}" >&2
  exit 1
fi
chmod +x "${tool}"

ARCH=x86_64 "${tool}" "${appdir}" "${output}"
echo "AppImage written to ${output}"
