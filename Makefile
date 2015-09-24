UNAME_S := $(shell uname -s)
TARGET = release;
 
ifneq ($(grep -qs Fedora /etc/redhat-release || echo 1), 1)
	USRPATH = /usr/local
	export PATH := $(USRPATH)/bin:$(PATH)
else ifeq ($(UNAME_S), Linux)
	USRPATH = /usr
else ifeq ($(UNAME_S), Darwin)
	USRPATH = /usr/local
endif

all:
ifeq ($(TARGET), release)
	$(USRPATH)/bin/cargo build --release
else
	# XXX This symlink is to fix a bug with building zmq crate
	mkdir -p $(shell pwd)/lib
	ln -s /usr/local/lib $(shell pwd)/lib/x86_64-unknown-linux-gnu
	$(USRPATH)/bin/cargo build
endif

install:
	install -m 0755 target/$(TARGET)/incli $(USRPATH)/bin

uninstall:
	rm -f $(USRPATH)/bin/incli

clean:
	$(USRPATH)/bin/cargo clean