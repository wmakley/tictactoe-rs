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
