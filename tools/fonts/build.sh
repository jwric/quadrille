#!/bin/sh
# Rebuild the fonts shipped in crates/quadrille/fonts from fonts/.
#
# Requires python3 with fontTools. Departure Mono ships as upstream's OTF;
# the derived faces are compiled from BDF sources in fonts/sources.
set -e
cd "$(dirname "$0")/../.."

out=crates/quadrille/fonts
tools=tools/fonts

cp fonts/departure-mono/DepartureMono-Regular.otf "$out/"
cp fonts/departure-mono/OFL.txt "$out/"

python3 "$tools/derive_tight.py" \
    fonts/sources/departure-mono-11.bdf fonts/sources/departure-mono-tight-11.bdf
python3 "$tools/bdf2ttf.py" \
    fonts/sources/departure-mono-tight-11.bdf "$out/DepartureMonoTight-Regular.ttf" \
    --family "Departure Mono Tight" --style Regular --em 11
python3 "$tools/license.py" "$out/DepartureMonoTight-Regular.ttf" \
    "Copyright 2022-2024 Helena Zhang (helenazhang.com). Departure Mono Tight is a derivative of Departure Mono."
