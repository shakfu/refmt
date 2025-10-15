
.PHONY: all build install clean test

all: build

build:
	@cargo build --release

test:
	@cargo test --workspace

clean:
	@rm -rf target

install: build
	@cp target/release/refmt /usr/local/bin/
	@echo "refmt installed"
