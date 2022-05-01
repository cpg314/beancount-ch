#!/usr/bin/env sh
set -euo pipefail

VERSION=$(tomlq -r ".package.version" Cargo.toml)
PACKAGE=$(tomlq -r ".package.name" Cargo.toml)
for ARCH in x86_64-unknown-linux-gnu 
do
    cross build -r --target $ARCH
    DEST=target/$PACKAGE-$VERSION-$ARCH.zip
    cargo about generate about.hbs > licenses.html
    echo $DEST
    zip -j -r $DEST target/$ARCH/release/$PACKAGE licenses.html
    zip -sf $DEST
done
