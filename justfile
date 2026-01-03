import 'docker.just'
import? 'private.just'
import? 'tests/hurl.just'

image_name := "ghcr.io/rezi-labs/sunday"
export LOCAL := "true"
export API_KEY_ONE := 'very-secure-key-one'
export API_KEY_TWO := 'very-secure-key-two'
export DATABASE_URL := 'postgresql://sunday:sunday_password@localhost:5432/sunday'
export CORS_ALLOWED_ORIGINS := '*'

run:
    just db-stop
    just db-start
    @echo "Waiting for database to be ready..."
    @sleep 3
    cargo run

watch:
    @echo "Starting PostgreSQL database..."
    just db-start
    @echo "Waiting for database to be ready..."
    @sleep 3
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

# Database commands
db-start:
    @echo "Starting PostgreSQL database..."
    docker compose -f docker-compose.dev.yml up -d

db-stop:
    @echo "Stopping PostgreSQL database..."
    docker compose -f docker-compose.dev.yml down

db-logs:
    docker compose -f docker-compose.dev.yml logs -f postgres

# Open test UI in browser
ui:
    @echo "Opening test UI in browser..."
    xdg-open test_ui/index.html || open test_ui/index.html || start test_ui/index.html
