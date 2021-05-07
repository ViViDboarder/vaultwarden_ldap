FROM rust:1.46

WORKDIR /usr/src/
RUN USER=root cargo new --bin vaultwarden_ldap
WORKDIR /usr/src/vaultwarden_ldap

# Compile dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build --locked --release

# Remove bins to make sure we rebuild
RUN rm ./target/release/deps/vaultwarden_ldap*
# Copy source and install
COPY src ./src
RUN cargo install --path .

CMD ["vaultwarden_ldap"]
