# Intecture

Intecture is a developer friendly, language agnostic configuration management tool for server systems.

* Extensible support for virtually any programming language
* Standard programming interface. No DSL. No magic.
* Rust API library (and bindings for popular languages)

You can find out more at [intecture.io](http://intecture.io).

# Install

## Auto

The quick way to get up and running is by using the Intecture installer.

```
$ curl -sSf https://static.intecture.io/install.sh | sh
```

## Manual

First, as this project is written in Rust, you'll need...well, [Rust!](https://www.rust-lang.org)

Next, clone this repository to your local machine and use the Makefile to build it:

```
$ git clone #...
$ cd intecture-cli/
$ make && sudo make install
```

Once this has finished, you should have a shiny new binary called *incli*, which lives in */usr/local/bin* if it exists, or */usr/bin* if not.

# Uninstall

Run the uninstall target on the Makefile:

```
$ cd intecture-cli/
$ sudo make uninstall
```

# Support

For enterprise support and consulting services, please email <mailto:support@intecture.io>.

For any bugs, feature requests etc., please ticket them on GitHub.