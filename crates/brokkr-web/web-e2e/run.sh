#!/usr/bin/env bash
# Run the full screenshot harness in one command: serve a release build of the
# console, then take every scene in the dark and the light theme. The exit code
# is not zero when a scene has a console error or a failed assert.
#
#   ./run.sh                      # all scenes, both themes, into shots/
#   ONLY=^session- ./run.sh       # only the scenes whose names match
#   PORT=9181 THEMES=light ./run.sh
#
# Needs: trunk, node, and `npm ci` in this folder (it installs the browser).
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
crate="$(dirname "$here")"
port="${PORT:-9080}"
themes="${THEMES:-dark light}"
out="${OUT_ROOT:-$here/shots}"
url="http://127.0.0.1:$port"

mkdir -p "$out"
if [ ! -d "$here/node_modules/@playwright/test" ]; then
  (cd "$here" && npm ci) || exit 1
fi

# trunk serve builds first; the harness starts when the page answers.
(cd "$crate" && exec trunk serve --release --port "$port" --no-autoreload) \
  >"$out/trunk.log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null; wait "$server" 2>/dev/null' EXIT

for _ in $(seq 1 900); do
  if curl -sf -o /dev/null "$url/"; then break; fi
  if ! kill -0 "$server" 2>/dev/null; then
    echo "trunk serve stopped; see $out/trunk.log" >&2
    tail -n 40 "$out/trunk.log" >&2
    exit 1
  fi
  sleep 1
done
curl -sf -o /dev/null "$url/" || { echo "no answer from $url" >&2; exit 1; }

status=0
for theme in $themes; do
  echo "== theme: $theme"
  (cd "$here" && URL="$url" THEME="$theme" OUT="$out/$theme" node shots.mjs) || status=1
done
exit "$status"
