.PHONY: all
all: release js

.PHONY: release
release:
	cargo build --release

.PHONY: js
js:
	cd svelte && pnpm run build
	rsync -av --delete svelte/build/ static/

.PHONY: docker-image
docker-image:
	docker build -t tictactoe-rs:latest .
