#!/bin/sh
docker run --rm \
    -v cargo_registry:/usr/local/cargo/registry \
    -v cargo_git:/usr/local/cargo/git \
    -v "$(pwd):/work" \
    -w=/work \
    -it \
    faldor20/rust-cross-comp-nightly:latest \
    bash