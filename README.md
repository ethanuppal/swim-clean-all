# swim-clean-all

![Crates.io Version](https://img.shields.io/crates/v/swim-clean-all)
![Crates.io License](https://img.shields.io/crates/l/swim-clean-all)

[`swim`](https://gitlab.com/spade-lang/swim) subcommand inspired by the amazing
[`cargo-clean-all`](https://github.com/dnlmlr/cargo-clean-all) for Rust
(although they probably have a better implementation than the hacky code I
wrote!).

## Showcase

![Example usage of the tool](./asset/showcase.gif)

## Install

Install from <crates.io>:

```
cargo install swim-clean-all
```

(Sorry, no `cargo binstall` magic yet.)

## Usage

```
$ swim clean-all --help
Usage: swim-clean-all [<search_root>] [--skip <skip...>] [--max-depth <max-depth>]

Recursively clean all swim projects in a given directory that match the specified criteria

Positional Arguments:
  search_root       the root directory to recursively search for swim projects;
                    defaults to the cwd

Options:
  --skip            directories to skip when traversing
  --max-depth       maximum depth search limit; defaults to 100
  --help, help      display usage information
```

Note that cleaning a project will erase the entire build folder instead of
calling `swim clean` -- I will add support for this customization if requested.
