#! /bin/sh

CARGO_VERSION=$(cargo pkgid --offline | sed 's/.*#//')
GIT_VERSION=${GIT_VERSION:-$(git describe --tags --exact-match)}
if ! [ "v$CARGO_VERSION" = "$GIT_VERSION" ]; then
    echo "ERROR: Cargo version (v$CARGO_VERSION) and git version ($GIT_VERSION) do not match"
    exit 1
fi
