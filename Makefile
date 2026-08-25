.PHONY: all build install uninstall clean test lint fmt publish publish-dry-run

# Override with: make install PREFIX=~/.local
PREFIX ?= /usr/local
BINDIR := $(PREFIX)/bin

all: build

build:
	@cargo build --release

test:
	@cargo test --workspace

lint:
	@cargo clippy --workspace --all-targets -- -D warnings

fmt:
	@cargo fmt --all

clean:
	@rm -rf target

install: build
	@install -d "$(BINDIR)"
	@install -m 755 target/release/reformat "$(BINDIR)/reformat"
	@echo "installed $(BINDIR)/reformat"

uninstall:
	@rm -f "$(BINDIR)/reformat"
	@echo "removed $(BINDIR)/reformat"

# Note: the `reformat` crate's dry run fails until `reformat-core` of the same
# version is on crates.io -- it cannot resolve a version that is not published
# yet. That is expected; `publish` below releases them in dependency order.
publish-dry-run:
	cargo publish --dry-run -p reformat-core
	cargo publish --dry-run -p reformat-plugins
	cargo publish --dry-run -p reformat

publish:
	cargo publish -p reformat-core
	@echo "waiting for crates.io to index reformat-core..."
	@sleep 15
	cargo publish -p reformat-plugins
	@echo "waiting for crates.io to index reformat-plugins..."
	@sleep 15
	cargo publish -p reformat
