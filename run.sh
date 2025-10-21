#!/usr/bin/env bash

VISUALS=false
unsafe=false

while [[ $# -gt 0 ]]; do
    case $1 in
        visuals)
            VISUALS=true
            shift
            ;;
        unsafe)
            UNSAFE=true
            shift
            ;;
        none)
            VISUALS=false
            UNSAFE=false
            shift
            ;;
        *)
            echo "unknown option: $1"
            shift
            ;;
    esac
done

FEATURES=""
if [ "$VISUALS" = true ] && [ "$UNSAFE" = true ]; then
    FEATURES=""
elif [ "$VISUALS" = true ]; then
    FEATURES="--no-default-features --features visuals"
elif [ "$UNSAFE" = true ]; then
    FEATURES="--no-default-features --features unsafe"
elif [ "$VISUALS" = false ] && [ "$UNSAFE" = false ]; then
    FEATURES="--no-default-features"
fi

if [ -z "$FEATURES" ]; then
    echo "running: cargo run --release"
    cargo run --release
else
    echo "running: cargo run --release $FEATURES"
    cargo run --release $FEATURES
fi
