
.PHONY: all build install clean test

all: build

build:
	@cargo build --release

test:
	@cargo test --workspace

clean:
	@rm -rf target

install: build
	@cp target/release/codeconvert /usr/local/bin/
	@echo "codeconvert installed"
