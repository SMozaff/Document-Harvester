# S-1 Locale Feasibility Spike

This nested workspace is a throwaway Linux demo. It is not imported by product crates and does not change `dh-app`, `dh-ui`, or installer definitions.

The demo pins Slint and `slint-build` to 1.19.0 and uses the product's Slint features (`std`, `backend-winit`, `renderer-software`, `compat-1-18`, and `accessibility`). `ui/main.slint` imports the repository's `data/validated_fonts/Vazirmatn-Regular.ttf` into the compiled spike binary and sets `Vazirmatn` as the default family.

From the repository root, build and run on a graphical Linux desktop:

```sh
cargo build --locked --manifest-path spike/s1/Cargo.toml
spike/s1/target/debug/s1-locale-spike
```

To reproduce the Xvfb interaction and screenshots, run:

```sh
bash spike/s1/check-linux.sh
```

The script launches one window at 720x520, captures English mode, increments the screen-state token twice, toggles Persian at runtime, then captures the same window with token 9. The script requires `Xvfb`, `xvfb-run`, `xdotool`, and `ffmpeg`.

The embedded-path fixture is intentionally a stress case. Its visual reordering in Persian mode is a finding, not a product-approved formatting technique. The standalone LTR path fixture is separate and should not be used to infer that arbitrary mixed text is safe.


## Revised S-1 composition

`ui/main-inline-failure.slint` preserves the original single-string stress fixture. `ui/main.slint` is the revised demo compiled by `build.rs`: it renders a Persian sentence prefix, the path, and its suffix as separate `Text` elements. `HorizontalLayout` keeps physical child order, so the Persian row places the suffix on the left, the standalone LTR path in the middle, and the sentence prefix on the right. Read right-to-left, this yields prefix → path → suffix. The complete path also appears in a separate full-width LTR text element. No bidi control characters are inserted.

Run `bash spike/s1/check-linux.sh` to capture the revised English and Persian states. Outputs are `docs/ui/s1/linux-rev2-720x520-english.png` and `docs/ui/s1/linux-rev2-720x520-persian-segmented-state-9.png`; the Persian screenshot is captured after two state increments and a locale switch in the same window.

The Linux check builds with `--locked`, then moves the source TTF aside while the binary runs. This verifies that the imported font is available from the compiled app rather than a runtime file. The script restores the TTF on exit.

### Package proposal (notes only; do not apply in this spike)

The demo embeds Vazirmatn at compile time, so the current proposal adds the OFL text to each package but does not add a loose TTF. If a future product build stops embedding it, package the font separately and configure its runtime loading.

Debian `packaging/linux/nfpm-deb.yaml`:

```yaml
  - src: packaging/licenses/Vazirmatn-OFL.txt
    dst: /usr/share/doc/document-harvester/Vazirmatn-OFL.txt
    file_info:
      mode: 0644
```

AppImage `packaging/linux/build-appimage.sh`:

```bash
install -m 0644 "${repo_root}/packaging/licenses/Vazirmatn-OFL.txt" \
  "${appdir}/usr/share/doc/document-harvester/Vazirmatn-OFL.txt"
```

Windows `packaging/windows/nfpm-msi.yaml`:

```yaml
  - src: packaging/licenses/Vazirmatn-OFL.txt
    dst: /Vazirmatn-OFL.txt
    type: license
    file_info:
      mode: 0644
```
