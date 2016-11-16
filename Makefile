TARGET = release
PREFIX = /usr/local

all:
ifeq ($(TARGET), release)
	cargo build --release
else
	cargo build
endif

install:
	install -m 0755 target/$(TARGET)/incli $(PREFIX)/bin

uninstall:
	rm -f $(PREFIX)/bin/incli

test:
ifeq ($(TARGET), release)
	cargo test --release
else
	cargo test
endif

clean:
	cargo clean
