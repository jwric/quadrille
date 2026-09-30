#!/usr/bin/env python3
"""Writes the copyright and SIL OFL 1.1 license records into a font's name table.

usage: license.py font.ttf "Copyright ..."
"""
import sys

from fontTools.ttLib import TTFont

OFL = (
    "This Font Software is licensed under the SIL Open Font License, Version 1.1. "
    "This license is available with a FAQ at: https://openfontlicense.org"
)


def main(path, copyright):
    font = TTFont(path)
    names = font["name"]
    for record, value in ((0, copyright), (13, OFL), (14, "https://openfontlicense.org")):
        names.setName(value, record, 3, 1, 0x409)
        names.setName(value, record, 1, 0, 0)
    font.save(path)


if __name__ == "__main__":
    main(*sys.argv[1:3])
