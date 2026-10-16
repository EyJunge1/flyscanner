.PHONY: demo frontend-sim web build release docker-demo docker-beast docker-full fmt lint check

demo: web
	cargo run -- --demo --lat 52.52 --lon 13.405 --db data/flyscanner.db

frontend-sim:
	cd frontend && npm install && npm run sim

web:
	cd frontend && npm install && npm run build

build: web
	cargo build

release: web
	cargo build --release

docker-demo:
	docker compose -f deploy/docker-compose.yml --profile demo up --build

docker-beast:
	docker compose -f deploy/docker-compose.yml --profile beast up --build

docker-full:
	docker compose -f deploy/docker-compose.yml -f deploy/docker-compose.sdr.yml --profile full up --build

fmt:
	cargo fmt --all
	cd frontend && npm run format

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cd frontend && npm run lint

check: lint
	cd frontend && npm run check
