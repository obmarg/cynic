.PHONY: rustup
rustup:
	rustup install stable
	rustup target add wasm32-unknown-unknown

.PHONY: js
js:
	cd js && npm install && npm run build

.PHONY: ci
ci: rustup js
	trunk build
