#!/bin/sh
# Builds the showcase for the browser into web/pkg.
#
#   web/build.sh           optimized build
#   web/build.sh --dev     unoptimized, faster to build
#
# Serve the web directory with any static server afterwards, for example:
#   python3 -m http.server 8080 --directory web
set -e
cd "$(dirname "$0")/.."

profile=--release
[ "$1" = "--dev" ] && profile=--dev

wasm-pack build demo $profile --target web --out-dir ../web/pkg --no-typescript --no-pack
