ARG BUILD_TAG=1.57.0
ARG RUN_TAG=slim-buster
FROM rust:${BUILD_TAG}-${RUN_TAG} as builder

WORKDIR /usr/src/
RUN USER=root cargo new --bin vaultwarden_ldap
WORKDIR /usr/src/vaultwarden_ldap

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Compile dependencies
RUN cargo build --locked --release

# Remove bins to make sure we rebuild
# hadolint ignore=DL3059
RUN rm ./target/release/deps/vaultwarden_ldap*
# Copy source and install
COPY src ./src
RUN cargo build --release

FROM rust:${BUILD_TAG}-${RUN_TAG}
WORKDIR /app
COPY --from=builder /usr/src/vaultwarden_ldap/target/release/vaultwarden_ldap /usr/local/bin/

CMD ["/usr/local/bin/vaultwarden_ldap"]
