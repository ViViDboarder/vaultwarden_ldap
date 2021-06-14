DOCKER_TAG ?= vaultwarden_ldap_${USER}

.PHONY: all
all: test check release

# Default make target will run tests
.DEFAULT_GOAL = test

# Build debug version
target/debug/vaultwarden_ldap: src/
	cargo build

# Build release version
target/release/vaultwarden_ldap: src/
	cargo build --locked --release

.PHONY: build
build: debug

.PHONY: debug
debug: target/debug/vaultwarden_ldap

.PHONY: release
release: target/release/vaultwarden_ldap

# Run debug version
.PHONY: run-debug
run-debug: target/debug/vaultwarden_ldap
	target/debug/vaultwarden_ldap

# Run all tests
.PHONY: test
test:
	cargo test

# Installs pre-commit hooks
.PHONY: install-hooks
install-hooks:
	pre-commit install --install-hooks

# Checks files for encryption
.PHONY: check
check:
	pre-commit run --all-files

# Checks that version matches the current tag
.PHONY: check-version
check-version:
	./scripts/check-version.sh

.PHONY: clean
clean:
	rm -f ./target

.PHONY: docker-build-all
docker-build-all: docker-build docker-build-alpine

.PHONY: docker-build
docker-build:
	docker build -f ./Dockerfile -t $(DOCKER_TAG) .

.PHONY: docker-build-alpine
docker-build-alpine:
	docker build -f ./Dockerfile.alpine -t $(DOCKER_TAG):alpine .
