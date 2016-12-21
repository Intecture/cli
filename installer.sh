#!/bin/sh
# Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
# top-level directory of this distribution and at
# https://intecture.io/COPYRIGHT.
#
# Licensed under the Mozilla Public License 2.0 <LICENSE or
# https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
# modified, or distributed except according to those terms.

# Undefined vars are errors
set -u

# Globals
prefix="{{prefix}}"
libdir="{{libdir}}"
libext="{{libext}}"
pkgconf="{{pkgconf}}"
pkgconfdir="{{pkgconfdir}}"
os="{{os}}"

do_install() {
    need_cmd $pkgconf

    local _one=
    local _two=

    if ! $($pkgconf --exists libzmq); then
        if [ "$os" = "darwin" ]; then
            _one="5"
            _two=$libext
        else
            _one=$libext
            _two="5"
        fi
        install -m 755 lib/libzmq.$libext $libdir/libzmq.$_one.$_two
        ln -s $libdir/libzmq.$_one.$_two $libdir/libzmq.$libext
        install -m 644 lib/pkgconfig/libzmq.pc $pkgconfdir
        install -m 644 include/zmq.h $prefix/include/

        if [ "$os" = "freebsd" ]; then
            install -m 644 lib/libstdc++.so.6 $libdir/
        fi
    fi

    if ! $($pkgconf --exists libczmq); then
        if [ "$os" = "darwin" ]; then
            _one="4"
            _two=$libext
        else
            _one=$libext
            _two="4"
        fi
        install -m 755 lib/libczmq.$libext $libdir/libczmq.$_one.$_two
        ln -s $libdir/libczmq.$_one.$_two $libdir/libczmq.$libext
        install -m 644 lib/pkgconfig/libczmq.pc $pkgconfdir
        install -m 644 include/czmq.h $prefix/include/
        install -m 644 include/czmq_library.h $prefix/include/
        install -m 644 include/czmq_prelude.h $prefix/include/
        install -m 644 include/zactor.h $prefix/include/
        install -m 644 include/zarmour.h $prefix/include/
        install -m 644 include/zauth.h $prefix/include/
        install -m 644 include/zbeacon.h $prefix/include/
        install -m 644 include/zcert.h $prefix/include/
        install -m 644 include/zcertstore.h $prefix/include/
        install -m 644 include/zchunk.h $prefix/include/
        install -m 644 include/zclock.h $prefix/include/
        install -m 644 include/zconfig.h $prefix/include/
        install -m 644 include/zdigest.h $prefix/include/
        install -m 644 include/zdir.h $prefix/include/
        install -m 644 include/zdir_patch.h $prefix/include/
        install -m 644 include/zfile.h $prefix/include/
        install -m 644 include/zframe.h $prefix/include/
        install -m 644 include/zgossip.h $prefix/include/
        install -m 644 include/zhash.h $prefix/include/
        install -m 644 include/zhashx.h $prefix/include/
        install -m 644 include/ziflist.h $prefix/include/
        install -m 644 include/zlist.h $prefix/include/
        install -m 644 include/zlistx.h $prefix/include/
        install -m 644 include/zloop.h $prefix/include/
        install -m 644 include/zmonitor.h $prefix/include/
        install -m 644 include/zmsg.h $prefix/include/
        install -m 644 include/zpoller.h $prefix/include/
        install -m 644 include/zproxy.h $prefix/include/
        install -m 644 include/zrex.h $prefix/include/
        install -m 644 include/zsock.h $prefix/include/
        install -m 644 include/zstr.h $prefix/include/
        install -m 644 include/zsys.h $prefix/include/
        install -m 644 include/zuuid.h $prefix/include/
    fi

    if [ -f "lib/libssl.$libext" ] && ! $($pkgconf --exists libssl); then
        case "$os" in
            "debian" | "ubuntu")
                install -m 755 lib/libssl.$libext $libdir/x86_64-linux-gnu/libssl.$libext.1.0.0
                ln -s $libdir/x86_64-linux-gnu/libssl.$libext.1.0.0 $libdir/x86_64-linux-gnu/libssl.$libext
                ;;
            *)
                install -m 755 lib/libssl.$libext $libdir
                ;;
        esac
    fi

    install -m 755 incli $prefix/bin
}

do_uninstall() {
	rm -f $prefix/bin/incli
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        echo "need '$1' (command not found)" >&2
        exit 1
    fi
}

main() {
	if [ $# -eq 0 ]; then
		echo "Usage: installer.sh <install|uninstall>"
		exit 0
	fi

	case "$1" in
		install)
			do_install
			;;

		uninstall)
			do_uninstall
			;;

		*)
			echo "Unknown option $1"
			exit 1
			;;
	esac
}

main "$@"
