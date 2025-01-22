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

# Run bootstrapped integration test
.PHONY: itest-up
itest-up:
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest.yml \
		build
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest.yml \
		up -d vaultwarden ldap

.PHONY: itest-run
itest-run:
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest.yml \
		run ldap_sync

.PHONY: itest-stop
itest-stop:
	docker compose stop

.PHONY: itest
itest: itest-up itest-run itest-stop

# Run bootstrapped integration test for anonymous bind
.PHONY: itest-up-anon
itest-up-anon:
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest-anon.yml \
		build
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest-anon.yml \
		up -d vaultwarden ldap ldap_admin

.PHONY: itest-run-anon
itest-run-anon:
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest-anon.yml \
		run --rm ldap_sync

.PHONY: itest-stop-anon
itest-stop-anon:
	docker compose stop

.PHONY: itest-anon
itest-anon: itest-up-anon itest-run-anon itest-stop-anon

# Run bootstrapped integration test using env for config
.PHONY: itest-env
itest-env:
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest-env.yml \
		build
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest-env.yml \
		up -d vaultwarden ldap
	docker compose -f docker-compose.yml \
		-f itest/docker-compose.itest-env.yml \
		run ldap_sync
	docker compose stop

.PHONY: clean-itest
clean-itest:
	docker compose down -v --remove-orphans

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
docker-build-all: docker-build

.PHONY: docker-build
docker-build:
	docker build -f ./Dockerfile -t $(DOCKER_TAG) .
