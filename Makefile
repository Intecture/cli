UNAME_S := $(shell uname -s)
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
	$(USRPATH)/bin/cargo build --release
else
	$(USRPATH)/bin/cargo build
endif

install:
	install -m 0755 target/$(TARGET)/incli $(USRPATH)/bin

uninstall:
	rm -f $(USRPATH)/bin/incli

clean:
	$(USRPATH)/bin/cargo clean