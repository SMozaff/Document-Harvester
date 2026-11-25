#!/usr/bin/env bash
# Smoke check for the .deb installer (D20). Extracts the installed file set
# into an empty location, asserts the binary / PDFium / license files, then
# runs the built-in self-test with no network namespace.
#
# A missing file, an unavailable network-isolation tool or a non-zero self-test
# exit fails this script. This script is code only; verification runs in CI.
set -euo pipefail

package="$1"
install_root="$2"

if [[ -z "${package}" || -z "${install_root}" ]]; then
  echo "usage: verify-deb.sh <package.deb> <empty-install-root>" >&2
  exit 2
fi

if ! command -v dpkg-deb >/dev/null 2>&1; then
  echo "dpkg-deb is required to verify the .deb" >&2
  exit 2
fi
if ! command -v unshare >/dev/null 2>&1; then
  echo "unshare is required to run the self-test without network access" >&2
  exit 2
fi

rm -rf "${install_root}"
mkdir -p "${install_root}"
dpkg-deb -x "${package}" "${install_root}"

app_dir="${install_root}/usr/lib/document-harvester"
doc_dir="${install_root}/usr/share/doc/document-harvester"
required=(
  "${app_dir}/dh-app"
  "${app_dir}/libpdfium.so"
  "${doc_dir}/SLINT-LICENSE.md"
  "${doc_dir}/PDFIUM-LICENSE.txt"
  "${doc_dir}/pdfium-licenses/pdfium.txt"
)
for file in "${required[@]}"; do
  if [[ ! -f "${file}" ]]; then
    echo "deb package is missing installed file: ${file}" >&2
    exit 1
  fi
done
if [[ ! -x "${app_dir}/dh-app" ]]; then
  echo "installed binary is not executable: ${app_dir}/dh-app" >&2
  exit 1
fi

echo "deb metadata:"
dpkg-deb -f "${package}" Package Version Architecture License

echo "running installed self-test with networking disabled"
# --pdfium-self-test resolves its fixtures from the compile-time repository
# path, so this check runs from the checkout that built the package.
unshare --net --fork -- bash -c 'cd "$1" && exec "$1/dh-app" --pdfium-self-test' _ "${app_dir}"

echo "deb install smoke check passed"
