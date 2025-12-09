# All is from https://github.com/extrawurst/gitui

.PHONY: docker-release, release-doc, release-client-mac-arm, release-client-mac-intel, release-client-win, release-linux-client-musl, release-linux-server-musl

build-client-release:
	cargo build --bin spa-client --release

build-spa-client-js:
	cd $(SPC_CLIENT_JS_DIR) && npm run build:release

# this is for local machine, not for GitHub Action
# make docker-release VERSION=1.2.3
docker-release:
ifeq ($(VERSION), )
	$(error VERSION is not set)
else
	DOCKER_BUILDKIT=1 docker build . -t="ghcr.io/fornetcode/spa-server:$(VERSION)"
	docker push ghcr.io/fornetcode/spa-server:$(VERSION)
endif

release-doc:
	set -e
	rm -fr docs/.vitepress/dist/*
	npm ci && npm run docs:build
	cd docs/.vitepress/dist
	git init
	git add -A
	git commit -m 'deploy'
	git push -f git@github.com:fornetcode/spa-server.git master:gh-pages
	cd -

release-client-mac-arm: build-client-release
	strip target/release/spa-client
	otool -L target/release/spa-client
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/spa-client-mac-aarch64.tar.gz ./spa-client
	ls -lisah ./release/spa-client-mac-aarch64.tar.gz

release-client-mac-intel: build-client-release
	strip target/release/spa-client
	otool -L target/release/spa-client
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/spa-client-mac-x86_64.tar.gz ./spa-client
	ls -lisah ./release/spa-client-mac-x86_64.tar.gz

release-client-win: build-client-release
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/spa-client-win.tar.gz ./spa-client.exe
	ls -l ./release/spa-client-win.tar.gz

release-linux-client-musl:
	cargo build --package spa-client --bin spa-client --target=x86_64-unknown-linux-musl --release
	strip target/x86_64-unknown-linux-musl/release/spa-client
	mkdir -p release
	tar -C ./target/x86_64-unknown-linux-musl/release/ -czvf ./release/spa-client-linux-musl.tar.gz ./spa-client

release-linux-server-musl:
	cargo build --package spa-server --bin spa-server --target=x86_64-unknown-linux-musl --release
	strip target/x86_64-unknown-linux-musl/release/spa-server
	mkdir -p release
	tar -C ./target/x86_64-unknown-linux-musl/release/ -czvf ./release/spa-server-linux-musl.tar.gz ./spa-server



