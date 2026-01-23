# vedtoob

This is a basic TUI browser for [Boot.dev](https://www.boot.dev/) courses.

## Installation

```sh
cargo install --git https://github.com/theodore-s-beers/vedtoob
```

Caveat: [`pandoc`](https://github.com/jgm/pandoc) is a runtime dependency, i.e., it needs to be in your `PATH`.

### Native TLS

`vedtoob` uses [`rustls`](https://github.com/rustls/rustls) by default. If `rustls` is unavailable on your platform (or if you prefer not to use it), there's also an option to build/install using your system's native TLS stack:

```sh
cargo install --git https://github.com/theodore-s-beers/vedtoob --no-default-features --features tls-native
```

## Usage

Now that this is a TUI, it should be self-explanatory. Just fire it up!

```sh
vedtoob
```
