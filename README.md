# swim-clean-all

[![Crates.io Version](https://img.shields.io/crates/v/swim-clean-all)](https://crates.io/crates/swim-clean-all)
![Crates.io License](https://img.shields.io/crates/l/swim-clean-all)

[`swim`](https://gitlab.com/spade-lang/swim) subcommand inspired by the amazing
[`cargo-clean-all`](https://github.com/dnlmlr/cargo-clean-all) for Rust
(although they probably have a better implementation than the hacky code I
wrote!).

## 🚀 Showcase

![Example usage of the tool](./asset/showcase.gif)

## ⚡️ Requirements

Make sure you have [`swim`](https://gitlab.com/spade-lang/swim) installed.
That's it!

## 📦 Install

Install from [crates.io](https://crates.io/crates/swim-clean-all):

```
cargo install swim-clean-all
```

(Sorry, no `cargo binstall` magic yet.)

## ✨ Usage

```
$ swim clean-all --help
Usage: swim-clean-all [<search_root>] [--skip <skip...>] [--max-depth <max-depth>] [--config <config>] [--ignore-config] [--verbose]

Recursively clean all swim projects in a given directory that match the specified criteria

Positional Arguments:
  search_root       the root directory to recursively search for swim projects;
                    defaults to the cwd

Options:
  --skip            directories to skip when traversing
  --max-depth       maximum depth search limit; defaults to 100
  --config          manually specify a config path, e.g., foo.toml
  --ignore-config   do not load and extend the config file
  --verbose         print debugging information
  --help, help      display usage information
```

Note that cleaning a project will erase the entire build folder instead of
calling `swim clean` -- I will add support for this customization if requested.
