.PHONY: all
all: release js

.PHONY: release
release:
	cargo build --release

.PHONY: js
js:
	cd svelte && pnpm run build
	cp -rv svelte/build/* static/
