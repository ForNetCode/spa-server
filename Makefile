# All is from https://github.com/extrawurst/gitui
SPC_CLIENT_JS_DIR = jsclient

.PHONY: build-spa-client-js, build-release, docker-release, release-doc

build-release:
	cargo build --bin spa-client --release

build-spa-client-js:
	cd $(SPC_CLIENT_JS_DIR) && npm run build:release

# this is for local machine, not for GitHub Action
# make docker-release VERSION=1.2.3
docker-release:
ifeq ($(VERSION), )
	$(error VEDRSION is not set)
else
	DOCKER_BUILDKIT=1 docker build . -t="ghcr.io/fornetcode/spa-server:$(VERSION)"
	docker push fornetcode/spa-server:$(VERSION)
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
