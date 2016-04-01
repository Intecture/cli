#!/bin/bash

curl -sSOL https://download.libsodium.org/libsodium/releases/libsodium-1.0.8.tar.gz
curl -sSOL https://download.libsodium.org/libsodium/releases/libsodium-1.0.8.tar.gz.sig
curl -sSOL https://download.libsodium.org/jedi.gpg.asc
gpg --import jedi.gpg.asc
gpg --verify libsodium-1.0.8.tar.gz.sig libsodium-1.0.8.tar.gz
tar zxf libsodium-1.0.8.tar.gz
cd libsodium-1.0.8
./configure
make
sudo make install
cd ..

git clone https://github.com/zeromq/czmq
cd czmq
./autogen.sh
./configure
make
sudo make install
cd ..
