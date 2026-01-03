hurl_opts := "--variables-file tests/variables --test"

build-and-tests: (up "-d") tests
    docker compose down

wait-for-api:
    hurl tests/health.hurl --retry 30 --retry-interval 1000 {{ hurl_opts }}

# run acceptance tests against the running test stack
tests *args: wait-for-api
    hurl tests/*.hurl {{ hurl_opts }} {{ args }}
