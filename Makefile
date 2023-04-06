.PHONY: js
js:
	cd svelte && pnpm run build
	cp -rv svelte/build/* static/
