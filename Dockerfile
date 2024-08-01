FROM rust:1.80 as builder

WORKDIR /usr/src/
RUN USER=root cargo new --bin vaultwarden_ldap
WORKDIR /usr/src/vaultwarden_ldap

# Compile dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build --locked --release

# Remove bins to make sure we rebuild
# hadolint ignore=DL3059
RUN rm ./target/release/deps/vaultwarden_ldap*
# Copy source and install
COPY src ./src
RUN cargo build --release

# Use most recent ubuntu LTS release
FROM ubuntu:24.04
RUN apt-get update \
    && apt-get install -y --no-install-recommends 'libssl-dev=3.*' \
    && rm -rf /var/cache/apt/lists
COPY --from=builder /usr/src/vaultwarden_ldap/target/release/vaultwarden_ldap /usr/local/bin/

CMD ["/usr/local/bin/vaultwarden_ldap"]
