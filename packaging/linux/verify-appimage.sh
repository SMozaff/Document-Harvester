#!/usr/bin/env bash
# Smoke check for the AppImage (D20). Copies the package into an empty
# location, unpacks it without FUSE, asserts the installed file set, then runs
# AppRun's self-test with no network namespace.
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

appimage="$1"
work_root="$2"

if [[ -z "${appimage}" || -z "${work_root}" ]]; then
  echo "usage: verify-appimage.sh <package.AppImage> <empty-work-root>" >&2
  exit 2
fi


repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
owner_pdf="${repo_root}/PDF_tests/01.pdf"
if [[ ! -f "${owner_pdf}" ]]; then
  echo "owner fixture is missing: ${owner_pdf}" >&2
  exit 2
fi

rm -rf "${work_root}"
mkdir -p "${work_root}"
cp "${appimage}" "${work_root}/document-harvester.AppImage"
chmod +x "${work_root}/document-harvester.AppImage"

cd "${work_root}"
./document-harvester.AppImage --appimage-extract > /dev/null

required=(
  "squashfs-root/AppRun"
  "squashfs-root/usr/bin/dh-app"
  "squashfs-root/usr/bin/libpdfium.so"
  "squashfs-root/usr/share/doc/document-harvester/SLINT-LICENSE.md"
  "squashfs-root/usr/share/doc/document-harvester/PDFIUM-LICENSE.txt"
  "squashfs-root/usr/share/doc/document-harvester/pdfium-licenses/pdfium.txt"
)
for file in "${required[@]}"; do
  if [[ ! -f "${file}" ]]; then
    echo "AppImage is missing unpacked file: ${file}" >&2
    exit 1
  fi
done
if [[ ! -x "squashfs-root/usr/bin/dh-app" ]]; then
  echo "AppImage binary is not executable: squashfs-root/usr/bin/dh-app" >&2
  exit 1
fi

chmod +x squashfs-root/AppRun squashfs-root/usr/bin/dh-app

echo "running unpacked AppImage self-test with networking disabled"
# Pass the owner fixture from the checkout explicitly: the installed AppImage
# must not depend on a build-time repository path.
run_without_network bash -c 'cd "$1" && exec "$1/AppRun" --pdfium-self-test "$2"' _ "${work_root}/squashfs-root" "${owner_pdf}"

echo "AppImage install smoke check passed"
