# {{crate_name}}

{{description}}

Part of the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem: zero-dependency, `no_std` by default.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add {{crate_name}}
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
