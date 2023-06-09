.PHONY: all
all: release js

.PHONY: deploy
deploy: js
	fly deploy

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

.PHONY: pnpm-ci
pnpm-ci:
	cd svelte && pnpm install --frozen-lockfile

.PHONY: ci
ci: pnpm-ci js
