.PHONY: dev-db migrate ensure-sqlx prepare build build-all test coverage lint clean \
       frontend-install frontend-dev frontend-build frontend-lint \
       e2e-install e2e-test docs

DEV_DB_URL := sqlite:./dev.db?mode=rwc

ensure-sqlx:
	@command -v cargo-sqlx >/dev/null 2>&1 || { echo "Installing sqlx-cli..."; cargo install sqlx-cli --no-default-features --features sqlite; }

dev-db: ensure-sqlx
	sqlx database create --database-url "$(DEV_DB_URL)"
	sqlx migrate run --source crates/storeit-db-sqlite/migrations --database-url "$(DEV_DB_URL)"

migrate: ensure-sqlx
	sqlx migrate run --source crates/storeit-db-sqlite/migrations --database-url "$(DEV_DB_URL)"

prepare: dev-db
	cargo sqlx prepare --workspace --database-url "$(DEV_DB_URL)"

build: prepare
	SQLX_OFFLINE=true cargo build --workspace --release

build-all: frontend-build build

test:
	cargo test --workspace

coverage:
	cargo llvm-cov --workspace --html --output-dir coverage/ --ignore-filename-regex 'main\.rs|oidc\.rs'
	cargo llvm-cov --workspace --fail-under-lines 93 --ignore-filename-regex 'main\.rs|oidc\.rs'

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

clean:
	cargo clean
	rm -f dev.db*

frontend-install:
	cd frontend && npm install

frontend-dev:
	cd frontend && npm run dev

frontend-build:
	cd frontend && npm run build

frontend-lint:
	cd frontend && npx tsc --noEmit

e2e-install:
	cd e2e && npm install && npx playwright install chromium webkit

e2e-test: build-all
	cd e2e && npm test

docs:
	mdbook build docs/user
	mdbook build docs/dev
