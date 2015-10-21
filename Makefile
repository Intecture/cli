UNAME_S := $(shell uname -s)
CARGO := $(shell which cargo)
TARGET = release
 
ifeq ($(UNAME_S), Linux)
	FEDORA := $(grep -qs Fedora /etc/redhat-release)
	ifeq ($$?, 0)
		USRPATH = /usr/local
		export PATH := $(USRPATH)/bin:$(PATH)
	else
		USRPATH = /usr
	endif
else ifeq ($(UNAME_S), Darwin)
	USRPATH = /usr/local
endif

all:
ifeq ($(TARGET), release)
	$(CARGO) build --release
else
	$(CARGO) build
endif

install:
	install -m 0755 target/$(TARGET)/incli $(USRPATH)/bin

uninstall:
	rm -f $(USRPATH)/bin/incli

test:
ifeq ($(TARGET), release)
	$(CARGO) test --release
else
	$(CARGO) test
endif

clean:
	$(CARGO) clean