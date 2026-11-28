#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
app="${repo_root}/spike/s1/target/debug/s1-locale-spike"
out_dir="${repo_root}/docs/ui/s1"

source "$HOME/.cargo/env"
for tool in cargo xvfb-run xdotool ffmpeg; do
  command -v "${tool}" >/dev/null || { echo "missing required tool: ${tool}" >&2; exit 2; }
done

cargo build --manifest-path "${repo_root}/spike/s1/Cargo.toml"
mkdir -p "${out_dir}"

cd "${repo_root}"
xvfb-run -a -s '-screen 0 1024x768x24' bash -c '
  set -euo pipefail
  app="$1"
  out_dir="$2"
  "$app" >/tmp/s1-locale-spike.log 2>&1 &
  app_pid=$!
  trap "kill \"$app_pid\" 2>/dev/null || true" EXIT

  window_id=""
  for _ in $(seq 1 50); do
    window_id=$(xdotool search --onlyvisible --name "S-1 Locale Feasibility" 2>/dev/null | head -1 || true)
    [[ -n "$window_id" ]] && break
    sleep 0.2
  done
  [[ -n "$window_id" ]] || { cat /tmp/s1-locale-spike.log; exit 1; }

  eval "$(xdotool getwindowgeometry --shell "$window_id")"
  printf "Window geometry: %sx%s at %s,%s\\n" "$WIDTH" "$HEIGHT" "$X" "$Y"
  [[ "$WIDTH" == 720 && "$HEIGHT" == 520 ]] || { echo "unexpected window size" >&2; exit 1; }
  xdotool mousemove 900 700
  sleep 0.3
  ffmpeg -loglevel error -y -f x11grab -video_size 720x520 -i "$DISPLAY+${X},${Y}" -frames:v 1 "$out_dir/linux-720x520-english.png"

  # Increment state twice in the existing window, then toggle locale.
  xdotool mousemove "$((X + 190))" "$((Y + 80))" click --repeat 2 --delay 150 1
  sleep 0.3
  xdotool mousemove "$((X + 530))" "$((Y + 80))" click 1
  sleep 0.4
  xdotool mousemove 900 700
  sleep 0.3
  ffmpeg -loglevel error -y -f x11grab -video_size 720x520 -i "$DISPLAY+${X},${Y}" -frames:v 1 "$out_dir/linux-720x520-persian-state-9.png"

  file "$out_dir/linux-720x520-english.png" "$out_dir/linux-720x520-persian-state-9.png"
' _ "${app}" "${out_dir}"
