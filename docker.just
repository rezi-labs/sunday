

build version="latest":
    docker build -t {{image_name}}:{{version}} .

up d="": build
    docker compose up {{d}}

down:
    docker compose down

push:
    just _d_push $(just get_version)

_d_push version:
    docker build -t {{image_name}}:{{version}} .
    docker push {{image_name}}:{{version}}

get_version:
    cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'
