# pacman-blame

CLI utility for querying info about the arch linux package database and displaying it in a nice way.

## Installation

### Prerequisites

Cargo, rust and (obviously) pacman (libalpm).

### Cargo

```console
$ git clone https://github.com/RagePly/pacman-blame.git
$ cd pacman-blame
$ cargo install --path . # installs pacman-blame to the default cargo bin/ path
```

## Examples

```console
$ pacman-blame -h                   # for help
$ pacman-blame -L -h                # help for the option -L
$ pacman-blame -L                   # list all packages
$ pacman-blame -Le                  # list all explicitly installed packages
$ pacman-blame -Lr package:glibc    # list all packages that has any dependency on glibc
$ pacman-blame -Ler gsfonts         # list all explicitly installed packages that depends on gsfonts
$ pacman-blame -Ld --format='%n %v' # exactly equal to pacman -Qd
```

## Format string

The format language used for the `--format` option.

| Format specifier | Replacement     |
| ---------------- | --------------- |
| `%n`             | package name    |
| `%c`             | package comment |
| `%v`             | package version |
| `%r`             | package reason  |
| `%%`             | literal '%'     |

The format specifier can also be, for example `%{n}`. This is to future proof the formatting language.
