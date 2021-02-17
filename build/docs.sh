#!/bin/bash
# run this script to generate docs for the build script.  Requires cargo-readme to be installed.
set -e
cd ..
cargo readme -i build/main.rs -o build/readme.md --no-title --no-license --no-indent-headings --no-badges

