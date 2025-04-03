# CS2 Offset Dumper — REWORKED EDITION

🔥 Fork of the original [cs2-dumper](https://github.com/a2x/cs2-dumper), redesigned for style, usability, and clarity. This version delivers a complete overhaul of the interface and functionality.

---

## 💡 What's New

- ✅ Fully interactive **checkbox menu** to select which file types to generate (`.cs`, `.json`, `.hpp`, `.rs`)
- 💨 Automatic **console clear** for a clean look before and after selections
- 🎨 **Styled output** with colorful tags: `[ OK ]`, `[ERR ]`, `[WARN]`, `[>]`
- 📦 Smarter defaults, fewer steps, less clutter
- ✍️ Custom ASCII art banner & enhanced UX
- 📝 Still powered by [`memflow`](https://github.com/memflow/memflow)

---

## 📦 File Output Options

You can choose any combination of:

- `cs`     → C# bindings
- `hpp`    → C++ header
- `json`   → structured raw dump
- `rs`     → Rust output

---

## 🚀 Usage

1. Run CS2 (main menu is enough).
2. Launch this tool.
3. Select desired output formats via interactive checkbox menu.
4. Done — results are saved in `/output`.

---

## ⚙️ Requirements

- Rust 1.74+
- Admin rights (on Windows) or `sudo` (on Linux) for low-level memory access

---

## 🛠 Example

```sh
cargo run --release
```

---

## 📄 License

MIT (c) 2024-2025  
Fork by [MyArchiveProjects](https://github.com/MyArchiveProjects)  
Original by [a2x](https://github.com/a2x/cs2-dumper)
