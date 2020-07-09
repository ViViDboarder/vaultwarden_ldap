FROM rust:1.33

WORKDIR /usr/src/
RUN USER=root cargo new --bin bitwarden_rs_ldap
WORKDIR /usr/src/bitwarden_rs_ldap

# Compile dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build --locked --release

# Remove bins to make sure we rebuild
RUN rm ./target/release/deps/bitwarden_rs_ldap*
# Copy source and install
COPY src ./src
RUN cargo install --path .

CMD ["bitwarden_rs_ldap"]
