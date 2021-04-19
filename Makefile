HUB ?= quay.io/isanchez
IMAGE ?= ossm-example-body-extension
VERSION ?= 0.0.1

build: wasm

wasm:
	cargo build --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/ossm_example_body_extension.wasm ./extension.wasm

.PHONY: clean
clean:
	rm extension.wasm || true
	rm -r build || true

.PHONY: container
image: clean build
	mkdir build
	cp container/manifest.yaml build/
	cp extension.wasm build/
	cd build && podman build -t ${HUB}/${IMAGE}:${VERSION} . -f ../container/Dockerfile

image.push: image
	podman push ${HUB}/${IMAGE}:${VERSION}
