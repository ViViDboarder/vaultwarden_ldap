ARG BUILD_TAG=1.57.0
ARG RUN_TAG=$BUILD_TAG

FROM rust:$BUILD_TAG as builder

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

FROM ubuntu:focal
WORKDIR /app
RUN apt update -y
RUN apt install -y libssl-dev --no-install-recommends
COPY --from=builder /usr/src/vaultwarden_ldap/target/release/vaultwarden_ldap /usr/local/bin/

CMD ["/usr/local/bin/vaultwarden_ldap"]
