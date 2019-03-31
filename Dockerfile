FROM rust:1.33

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

CMD ["bitwarden_rs_ldap"]
