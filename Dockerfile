FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.6.1 AS xx
FROM --platform=$BUILDPLATFORM rust:1.89-trixie AS builder
COPY --from=xx / /

WORKDIR /usr/src/vaultwarden_ldap

RUN apt-get update \
    && apt-get install -y --no-install-recommends clang=1:19.* lld=1:19.* \
    && rm -rf /var/cache/apt/lists

ARG TARGETPLATFORM

# Install target-specific system dependencies
RUN xx-apt-get install -y --no-install-recommends xx-c-essentials pkgconf libssl-dev \
    && rm -rf /var/cache/apt/lists

# Build
COPY . ./
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/src/vaultwarden_ldap/target,sharing=locked \
    <<BUILD_SCRIPT
#!/bin/sh -eu
PKG_CONFIG="$(command -v "$(xx-info)-pkg-config")" xx-cargo build --release
xx-verify "./target/$(xx-cargo --print-target-triple)/release/vaultwarden_ldap"
mkdir -p ./build
cp "./target/$(xx-cargo --print-target-triple)/release/vaultwarden_ldap" ./build
BUILD_SCRIPT

# Use a base Linux distro with a shell, glibc, and minimal TLS library packages:
# https://images.chainguard.dev/directory/image/wolfi-base/sbom
# Because the base image already has every package our binary needs, we don't need
# any further RUN instructions and can get away with not setting up QEMU during
# multi-platform builds
# hadolint ignore=DL3007
FROM chainguard/wolfi-base:latest
COPY --from=builder /usr/src/vaultwarden_ldap/build/vaultwarden_ldap /usr/local/bin/

CMD ["/usr/local/bin/vaultwarden_ldap"]
