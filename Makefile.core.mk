FINDFILES=find . \( -path ./.git -o -path ./out -o -path ./.github -o -path ./vendor -o -path ./frontend/node_modules \) -prune -o -type f
XARGS=xargs -0 -r
RELEASE_LDFLAGS='-extldflags -static -s -w'
BINARIES=./cmd/api ./cmd/envsubst

lint-markdown:
	@${FINDFILES} -name '*.md' -print0 | ${XARGS} mdl --ignore-front-matter --style .mdl.rb

lint: lint-markdown
	cargo clippy --tests --bins

check: lint
	cargo check

cve-check:
	cargo deny check advisories

license-check:
	cargo deny check licenses

fix:
	cargo clippy --fix --allow-staged --allow-dirty
	cargo fmt

format:
	cargo fmt

.PHONY: default
default: build

build:
	@DRY_RUN=1 scripts/build.sh

release:
	@scripts/build.sh
	
test:
	RUST_BACKTRACE=1 cargo test --tests --bins

clean:
	cargo clean

check-clean-repo:
	@scripts/check_clean_repo.sh

presubmit: test lint format check-clean-repo
