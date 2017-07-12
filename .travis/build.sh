#!/bin/sh

set -x
set -e

BUNDLE="gtk-3.18.1-2"
WD="$PWD"
cd "$HOME"
git clone https://github.com/gkoz/gtk-bootstrap.git
cd gtk-bootstrap
./bootstrap.sh "$WD/.travis/manifest.txt"
cd "$WD"
export PKG_CONFIG_PATH="$HOME/local/lib/pkgconfig"

travis-cargo build
