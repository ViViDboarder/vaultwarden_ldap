FROM ekidd/rust-musl-builder AS builder

RUN USER=rust cargo init

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release

# Remove temp src
RUN rm src/*.rs
COPY ./src ./src
# Fix permissions on source
RUN sudo chown -R rust:rust /home/rust/src/src

RUN touch ./src/main.rs
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/bitwarden_rs_ldap \
    /usr/local/bin/

CMD /usr/local/bin/bitwarden_rs_ldap
