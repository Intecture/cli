#!/bin/sh
# Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
# top-level directory of this distribution and at
# https://intecture.io/COPYRIGHT.
#
# Licensed under the Mozilla Public License 2.0 <LICENSE or
# https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
# modified, or distributed except according to those terms.

# Undefined vars are errors
set -u

# Globals
prefix=
libdir=
libext=
pkgconf="pkg-config"
pkgconfdir=
os="$(uname -s)"
make="make"

case "$os" in
    Linux)
        prefix="/usr"
        libext="so"

        # When we can statically link successfully, we should be able
        # to produce vendor-agnostic packages.
        if [ -f "/etc/centos-release" ]; then
            os="centos"
            libdir="$prefix/lib64"
        elif [ -f "/etc/fedora-release" ]; then
            os="fedora"
            libdir="$prefix/lib64"
        elif [ -f "/etc/lsb-release" ]; then
            os="ubuntu"
            libdir="$prefix/lib"
        elif [ -f "/etc/debian_version" ]; then
            os="debian"
            libdir="$prefix/lib"
        else
            echo "unsupported Linux flavour" >&2
            exit 1
        fi

        pkgconfdir="$libdir/pkgconfig"
        ;;

    FreeBSD)
        os="freebsd"
        prefix="/usr/local"
		libdir="$prefix/lib"
        pkgconf="pkgconf"
        pkgconfdir="$prefix/libdata/pkgconfig"
        libext="so"
        make="gmake"
        ;;

    Darwin)
        os="darwin"
        prefix="/usr/local"
		libdir="$prefix/lib"
        pkgconfdir="$libdir/pkgconfig"
        libext="dylib"
        ;;

    *)
        echo "unrecognized OS type: $os" >&2
        exit 1
        ;;
esac

main() {
    local _cargodir=$(pwd)
    local _tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t intecture)"
    cd "$_tmpdir"

    # ZeroMQ dependency
    if ! $($pkgconf --exists libzmq) || [ $($pkgconf libzmq --modversion) != "4.2.0" ]; then
        curl -sSOL https://github.com/zeromq/libzmq/releases/download/v4.2.0/zeromq-4.2.0.tar.gz
        tar zxf zeromq-4.2.0.tar.gz
        cd zeromq-4.2.0
        ./autogen.sh
        ./configure --prefix=$prefix --libdir=$libdir --with-pkgconfigdir=$pkgconfdir
        $make
        $make install
        cd ..
    fi

    # CZMQ dependency
    if ! $($pkgconf --exists libczmq) || [ $($pkgconf libczmq --modversion) != "4.0.1" ]; then
        curl -sSOL https://github.com/zeromq/czmq/releases/download/v4.0.1/czmq-4.0.1.tar.gz
        tar zxf czmq-4.0.1.tar.gz
        cd czmq-4.0.1
        ./configure --prefix=$prefix --libdir=$libdir --with-pkgconfigdir=$pkgconfdir
        $make
        $make install
        cd ..
    fi

    # OpenSSL dependency
    if ! $($pkgconf --exists openssl); then
        case "$os" in
            "redhat")
                yum install -y openssl-devel
                ;;
            "debian")
                apt-get install -y libssl-dev
                ;;
            "freebsd")
                pkg install -y openssl-devel
                ;;
        esac
    fi

    # Build and install project assets
    cargo build --release --manifest-path "$_cargodir/Cargo.toml"

    local _version=$($_cargodir/target/release/incli --version)
    local _pkgdir="incli-$_version"

    # Create package dir structure
    mkdir "$_pkgdir"
    mkdir "$_pkgdir/include"
    mkdir "$_pkgdir/lib"
    mkdir "$_pkgdir/lib/pkgconfig"

    # Project assets
    cp "$_cargodir/target/release/incli" "$_pkgdir"

    # ZeroMQ assets
    cp "$libdir/libzmq.$libext" "$_pkgdir/lib/"
    cp "$pkgconfdir/libzmq.pc" "$_pkgdir/lib/pkgconfig/"
    cp "$prefix/include/zmq.h" "$_pkgdir/include/"

    # CZMQ assets
    cp "$libdir/libczmq.$libext" "$_pkgdir/lib/"
    cp "$pkgconfdir/libczmq.pc" "$_pkgdir/lib/pkgconfig/"
    cp "$prefix/include/czmq.h" "$_pkgdir/include/"
    cp "$prefix/include/czmq_library.h" "$_pkgdir/include/"
    cp "$prefix/include/czmq_prelude.h" "$_pkgdir/include/"
    cp "$prefix/include/zactor.h" "$_pkgdir/include/"
    cp "$prefix/include/zarmour.h" "$_pkgdir/include/"
    cp "$prefix/include/zauth.h" "$_pkgdir/include/"
    cp "$prefix/include/zbeacon.h" "$_pkgdir/include/"
    cp "$prefix/include/zcert.h" "$_pkgdir/include/"
    cp "$prefix/include/zcertstore.h" "$_pkgdir/include/"
    cp "$prefix/include/zchunk.h" "$_pkgdir/include/"
    cp "$prefix/include/zclock.h" "$_pkgdir/include/"
    cp "$prefix/include/zconfig.h" "$_pkgdir/include/"
    cp "$prefix/include/zdigest.h" "$_pkgdir/include/"
    cp "$prefix/include/zdir.h" "$_pkgdir/include/"
    cp "$prefix/include/zdir_patch.h" "$_pkgdir/include/"
    cp "$prefix/include/zfile.h" "$_pkgdir/include/"
    cp "$prefix/include/zframe.h" "$_pkgdir/include/"
    cp "$prefix/include/zgossip.h" "$_pkgdir/include/"
    cp "$prefix/include/zhash.h" "$_pkgdir/include/"
    cp "$prefix/include/zhashx.h" "$_pkgdir/include/"
    cp "$prefix/include/ziflist.h" "$_pkgdir/include/"
    cp "$prefix/include/zlist.h" "$_pkgdir/include/"
    cp "$prefix/include/zlistx.h" "$_pkgdir/include/"
    cp "$prefix/include/zloop.h" "$_pkgdir/include/"
    cp "$prefix/include/zmonitor.h" "$_pkgdir/include/"
    cp "$prefix/include/zmsg.h" "$_pkgdir/include/"
    cp "$prefix/include/zpoller.h" "$_pkgdir/include/"
    cp "$prefix/include/zproxy.h" "$_pkgdir/include/"
    cp "$prefix/include/zrex.h" "$_pkgdir/include/"
    cp "$prefix/include/zsock.h" "$_pkgdir/include/"
    cp "$prefix/include/zstr.h" "$_pkgdir/include/"
    cp "$prefix/include/zsys.h" "$_pkgdir/include/"
    cp "$prefix/include/zuuid.h" "$_pkgdir/include/"

    # OpenSSL assets
    case "$os" in
        "debian" | "ubuntu" )
            cp "$libdir/x86_64-linux-gnu/libssl.$libext" "$_pkgdir/lib/"
            ;;

        "darwin" )
            ;;

        *)
            cp "$libdir/libssl.$libext" "$_pkgdir/lib/"
            ;;
    esac

    if [ "$os" = "freebsd" ]; then
        # GCC libc++ (FreeBSD only)
        # XXX Version is hardcoded...bleh!
        cp "$libdir/gcc49/libstdc++.so.6" "$_pkgdir/lib/"

        # libcrypto
        cp "$libdir/libcrypto.$libext" "$_pkgdir/lib/"
    fi

    # Configure installer.sh paths
    sed "s~{{prefix}}~$prefix~" < "$_cargodir/installer.sh" |
    sed "s~{{libdir}}~$libdir~" |
    sed "s~{{libext}}~$libext~" |
    sed "s~{{pkgconf}}~$pkgconf~" |
    sed "s~{{pkgconfdir}}~$pkgconfdir~" |
    sed "s~{{os}}~$os~" > "$_pkgdir/installer.sh"
    chmod u+x "$_pkgdir/installer.sh"

    local _pkgstoredir="$_cargodir/.pkg/$os"
    mkdir -p "$_pkgstoredir"

    local _tarball="$_pkgstoredir/$_pkgdir.tar.bz2"
    tar -cjf "$_tarball" "$_pkgdir"

    cd "$_cargodir"
}

main || exit 1
