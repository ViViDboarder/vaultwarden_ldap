FROM rust:1.33

WORKDIR /usr/src/
RUN USER=root cargo new --bin bitwarden_rs_ldap
WORKDIR /usr/src/bitwarden_rs_ldap

# Compile dependencies
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
# Remove temp src
RUN rm src/*.rs

# Copy source and install
COPY ./src ./src
RUN rm ./target/release/deps/bitwarden_rs_ldap*
RUN cargo install --path .

CMD ["bitwarden_rs_ldap"]
