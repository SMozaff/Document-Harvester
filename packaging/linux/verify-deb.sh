#!/usr/bin/env bash
# Smoke check for the .deb installer (D20). Extracts the installed file set
# into an empty location, asserts the binary / PDFium / license files, then
# runs the built-in self-test with no network namespace.
#
# A missing file, an unavailable network-isolation tool or a non-zero self-test
# exit fails this script. This script is code only; verification runs in CI.
set -euo pipefail

# Run a command with no network namespace. Ubuntu 24.04 runners restrict
# unprivileged user namespaces, so try an unprivileged namespace first, then
# passwordless sudo. Fail explicitly if neither works.
run_without_network() {
  if unshare --user --map-root-user --net true >/dev/null 2>&1; then
    unshare --user --map-root-user --net --fork -- "$@"
  elif sudo -n true >/dev/null 2>&1; then
    sudo -n unshare --net --fork -- "$@"
  else
    echo "network isolation unavailable: need unprivileged user namespaces or passwordless sudo" >&2
    exit 2
  fi
}

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

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
owner_pdf="${repo_root}/PDF_tests/01.pdf"
if [[ ! -f "${owner_pdf}" ]]; then
  echo "owner fixture is missing: ${owner_pdf}" >&2
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
# Pass the owner fixture from the checkout explicitly: the installed binary
# must not depend on a build-time repository path.
run_without_network bash -c 'cd "$1" && exec "$1/dh-app" --pdfium-self-test "$2"' _ "${app_dir}" "${owner_pdf}"

echo "deb install smoke check passed"
