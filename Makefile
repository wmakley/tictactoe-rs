.PHONY: all
all: release js

.PHONY: release
release:
	cargo build --release

.PHONY: js
js:
	cd svelte && pnpm run build
	rm -rf static/_app
	cp -rv svelte/build/* static/

.PHONY: docker-image
docker-image:
	docker build -t tictactoe-rs:latest .
