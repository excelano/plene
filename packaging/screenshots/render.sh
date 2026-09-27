#!/bin/sh
# Renders the window's screenshots for excelano.com/plene/, light and dark, and
# writes them to OUT-DIR at the page's size: 2000 pixels wide, quantized.
#
#     packaging/screenshots/render.sh ~/excelano.com/plene/img
#
# Needs a GPU adapter wgpu can open (a software one does), ImageMagick and
# pngquant.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
out="${1:?usage: render.sh OUT-DIR}"
[ -d "$out" ] || { echo "render.sh: no directory $out" >&2; exit 1; }
for tool in magick pngquant; do
    command -v "$tool" >/dev/null || { echo "render.sh: needs $tool" >&2; exit 1; }
done

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT INT TERM

cargo run --quiet --release --manifest-path "${here}/Cargo.toml" -- \
    "${here}/config.rs" "$stage"
for theme in light dark; do
    magick "${stage}/plene-gui-${theme}.png" -resize 2000x -strip "${stage}/web-${theme}.png"
    pngquant --quality 80-95 --speed 1 --force \
        --output "${out}/plene-gui-${theme}.png" "${stage}/web-${theme}.png"
    echo "wrote ${out}/plene-gui-${theme}.png"
done
