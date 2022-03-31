# All is from https://github.com/extrawurst/gitui

build-release:
	cargo build --release

release-mac: build-release
	strip target/release/spa-client
	otool -L target/release/spa-client
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/spa-client-mac.tar.gz ./spa-client
	ls -lisah ./release/spa-client-mac.tar.gz

release-win: build-release
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/spa-client-win.tar.gz ./spa-client.exe
	cargo install cargo-wix
	cargo wix --no-build --nocapture --output ./release/spa-client.msi
	ls -l ./release/spa-client.msi

release-linux-musl: build-linux-musl-release
	strip target/x86_64-unknown-linux-musl/release/spa-client
	mkdir -p release
	tar -C ./target/x86_64-unknown-linux-musl/release/ -czvf ./release/spa-client-linux-musl.tar.gz ./spa-client

build-linux-musl-release:
	cargo build --release --target=x86_64-unknown-linux-musl

docker-release:
	ifndef VERSION
	$(error VERSION is not set)
	endif
	DOCKER_BUILDKIT=1 docker build . -t="timzaak/spa-server:$(VERSION)"
	docker push timzaak/spa-server:$(VERSION)

