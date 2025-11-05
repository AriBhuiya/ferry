# Ferry

**Ferry** is a fast, modern, and secure file transfer tool — inspired by `scp`, but designed for today's networks.  
Built in Rust 🦀, Ferry aims to make peer-to-peer transfers *blazing fast, resumable, and discoverable* with a simple CLI.

> ⚠️ **Work in Progress** — this is an experimental prototype.  
> Being actively worked on. Not all functionalities work.

---

## ✨ Current status

✅ **Implemented**
- Basic **`serve`** command (`ferry serve`) (Mock server for discovery)
- Server **discovery** (`ferry discover`) using `mdns-sd` multicast
- Auto-generated random server names (`abrasive-bread`, `trite-metal`, etc.)

🚧 **In progress**
- File chunking & transfer pipeline
- Resume / integrity verification
- Secure pairing codes
- Adaptive concurrency and bandwidth control
- End-to-end encryption

---

## 🧭 Usage

### Run a Ferry server:

```bash
ferry serve
```
By default:

- Listens on 127.0.0.1:3625

- Uses the current directory (.) as the transfer root

- Auto-generates a friendly server name

Options:
```bash
ferry serve -H 0.0.0.0 -p 3625 --dir ~/Downloads --name myhost
```
### Discover Ferry Servers
```bash
ferry discover
```
```text
Discovered 2 services
┌────────────────┬─────────────────────────────┬───────────────┬──────┐
│ NAME           │ HOST                        │ ADDRESS       │ PORT │
╞════════════════╪═════════════════════════════╪═══════════════╪══════╡
│ abrasive-bread │ abrasive-bread.ferry.local. │ 172.31.32.1   │ 3625 │
│ trite-metal    │ trite-metal.ferry.local.    │ 172.31.42.191 │ 3625 │
└────────────────┴─────────────────────────────┴───────────────┴──────┘
```
Use -a to list all addresses and -i to adjust the discovery interval:
```bash
ferry discover -a -i 100
```

## 🦀 Building from source
```bash
git clone https://github.com/aribhuiya/ferry
cd ferry-rs
cargo build
```
Then:
```bash
cargo run -- serve
cargo run -- discover
```