# S-1 Locale Feasibility Spike

This nested workspace is a throwaway Linux demo. It is not imported by product crates and does not change `dh-app`, `dh-ui`, or installer definitions.

The demo pins Slint and `slint-build` to 1.19.0 and uses the product's Slint features (`std`, `backend-winit`, `renderer-software`, `compat-1-18`, and `accessibility`). `ui/main.slint` imports the repository's `data/validated_fonts/Vazirmatn-Regular.ttf` into the compiled spike binary and sets `Vazirmatn` as the default family.

From the repository root, build and run on a graphical Linux desktop:

```sh
cargo build --manifest-path spike/s1/Cargo.toml
spike/s1/target/debug/s1-locale-spike
```

To reproduce the Xvfb interaction and screenshots, run:

```sh
bash spike/s1/check-linux.sh
```

The script launches one window at 720x520, captures English mode, increments the screen-state token twice, toggles Persian at runtime, then captures the same window with token 9. The script requires `Xvfb`, `xvfb-run`, `xdotool`, and `ffmpeg`.

The embedded-path fixture is intentionally a stress case. Its visual reordering in Persian mode is a finding, not a product-approved formatting technique. The standalone LTR path fixture is separate and should not be used to infer that arbitrary mixed text is safe.
