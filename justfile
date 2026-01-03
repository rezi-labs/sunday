import 'docker.just'
import? 'private.just'
import? 'tests/hurl.just'

image_name := "ghcr.io/rezi-labs/sunday"
export LOCAL := "true"
export API_KEY_ONE := 'very-secure-key-one'
export API_KEY_TWO := 'very-secure-key-two'

run:
    cargo run

watch:
    cargo watch -x run

verify: lint test build-and-tests

test:
    cargo test

lint:
    cargo fmt --all -- --check
    cargo clippy

fmt:
    cargo fmt
    cargo fix --allow-dirty --allow-staged

# Update all dependencies to their latest versions
update-deps:
    cargo update

upgrade-deps:
    cargo upgrade
    @echo "Running cargo update to lock new versions..."
    cargo update

# Database migrations
db-setup:
    @echo "Setting up database with pgvector extension..."
    sqlx database create
    sqlx migrate run

db-migrate:
    sqlx migrate run

db-reset:
    sqlx database drop -y
    sqlx database create
    sqlx migrate run
