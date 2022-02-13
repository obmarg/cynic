TRUNK_VERSION = v0.14.0

.PHONY: trunk
trunk:
	wget -qO- https://github.com/thedodd/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-

.PHONY: js
js:
	cd js && npm install && npm run build

.PHONY: ci
ci: trunk js
	./trunk build
