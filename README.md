# swim-clean-all

[`swim`](https://gitlab.com/spade-lang/swim) subcommand inspired by the amazing
[`cargo-clean-all`](https://github.com/dnlmlr/cargo-clean-all) for Rust
(although they probably have a better implementation than the hacky code I
wrote!).


## Install


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
