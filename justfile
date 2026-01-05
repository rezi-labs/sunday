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

# Test Azure OpenAI API connection
test-azure:
    curl -X POST "https://switzerlandnorth.api.cognitive.microsoft.com/openai/deployments/hive-gpt-4.1-nano/chat/completions?api-version=2025-01-01-preview" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer ${AZURE_API_KEY}" \
        -d '{"messages":[{"role":"user","content":"I am going to Paris, what should I see?"}],"max_completion_tokens":13107,"temperature":1,"top_p":1,"frequency_penalty":0,"presence_penalty":0,"model":"hive-gpt-4.1-nano"}'

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

db-delete-volume:
    @echo "Stopping PostgreSQL database..."
    docker compose -f docker-compose.dev.yml down
    @echo "Deleting PostgreSQL volume..."
    docker volume rm sunday_postgres_data
