# cs2-dumper (modified fork)

Fork of the original [cs2-dumper](https://github.com/a2x/cs2-dumper) with a cleaner structure, reduced number of files, and additional dumping options.

This version introduces:
- Cleaner file organization
- Optional output selection:
  - `.cs` files only
  - `.cs` + `.json`
- Small internal improvements and fixes

Supports both Windows & Linux, powered by [memflow](https://github.com/memflow/memflow).

For Linux users, refer to the original [linux branch](https://github.com/a2x/cs2-dumper/tree/linux) (may be outdated).

---

## Getting Started

Download latest release or compile manually. Requires Rust 1.74.0+.

### Usage

1. Start CS2 (main menu is enough).
2. Run the `cs2-dumper` executable.

By default, it uses `memflow-native`. To use another memflow connector, pass the connector name and optional args.

Example for pcileech:
```
cs2-dumper -c pcileech -a :device=FPGA -vv
```

Some connectors (like `kvm`, `pcileech`, or `winio`) require admin/root privileges.

### Arguments

- `-c, --connector <connector>`: Memflow connector.
- `-a, --connector-args <args>`: Arguments for connector.
- `-f, --file-types <types>`: What to generate. Default: `cs`, `hpp`, `json`, `rs`.
- `-i, --indent-size <n>`: Spaces per indent. Default: 4.
- `-o, --output <dir>`: Output directory. Default: `output`.
- `-p, --process-name <name>`: Game process name. Default: `cs2.exe`.
- `-v...`: Verbose logging (can be stacked).
- `-h, --help`: Show help.
- `-V, --version`: Show version.

### Tests

Run basic tests with:
```
cargo test -- --nocapture
```

---

## License

MIT — see original [LICENSE](./LICENSE)
