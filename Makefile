
.PHONY: all build install clean

all: build

build:
	@cargo build --release

clean:
	@rm -rf target

install: build
	@cp target/release/codeconvert /usr/local/bin/
	@echo "codeconvert installed"
