import 'docker.just'
import? 'private.just'

image_name := "ghcr.io/rezi-labs/sunday"
export LOCAL := "true"

export POSTGRES_PASSWORD := "postgres"
export POSTGRES_USER := "postgres"
export POSTGRES_DB := "sunday"
export ADMIN_USERNAME :='admin'
export ADMIN_PASSWORD :='admin'
export RESET_ADMIN_USER := 'false'
export TENANT_API_KEY  := 'very-secure'

docker:
    docker compose up

run: start-db
    cargo run

watch:
    cargo watch -x run

verify: lint test

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

generate-session-secret:
    openssl rand -base64 64


# Start PostgreSQL database with docker-compose (recommended)
start-db: stop-db
    docker compose up -d postgres

# Start PostgreSQL database with persistent volume (alternative)
start-db-docker: stop-db
    docker run -d \
        --name sunday_postgres \
        -e POSTGRES_PASSWORD={{POSTGRES_PASSWORD}} \
        -e POSTGRES_USER={{POSTGRES_USER}} \
        -e POSTGRES_DB={{POSTGRES_DB}} \
        -p 5500:5432 \
        -v sunday-postgres-data:/var/lib/postgresql \
        postgres:18-alpine

# Stop PostgreSQL database
stop-db:
    docker compose down postgres || true
    docker stop sunday-postgres || true
    docker rm sunday-postgres || true

# Wipe PostgreSQL database volume (removes all data)
wipe-db-volume: stop-db
    docker compose down -v || true
    docker volume rm sunday-postgres-data || true
    docker volume rm sunday_sunday-postgres-data || true
