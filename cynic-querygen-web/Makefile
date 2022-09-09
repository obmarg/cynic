.PHONY: rustup
rustup:
	rustup install stable
	rustup target add wasm32-unknown-unknown
	
.PHONY: trunk
trunk:
	cargo install --locked trunk@0.16.0 || true

.PHONY: js
js:
	cd js && npm install && npm run build

.PHONY: ci
ci: rustup trunk js
	trunk build
