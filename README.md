# labwatch

A Rust CLI that shows real-time health status (CPU, memory, up/down) for every
machine in a home-lab cluster, in one live terminal dashboard.

## Why this project

- Fills the "systems/infra" gap in a portfolio that already covers AI (Scrappy),
  full-stack web (ITWS 2110, TimeTrack), and Java OSS (LangChain4j).
- Rust was chosen over Go because it builds directly on existing C/C++
  ownership/memory intuition, and its compiler-enforced safety around shared
  state is a real advantage for a tool polling multiple machines concurrently.
- Uses hardware already on hand (the home-lab fleet of old laptops + networking
  gear) rather than needing new infrastructure.
- Later, this tool's `--json` output could feed Scrappy's planned
  hardware/workload-aware routing — without coupling the two projects today.

## What v1 does

Run `labwatch` with a list of hostnames in a config file. It shows a live,
auto-refreshing terminal table: one row per node, with name, up/down status,
CPU %, and memory %, color-coded by health.

## Setup

1. Install Rust via rustup (not your OS package manager):
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   This installs `cargo` (build tool), `rustc` (compiler), and `rustup`
   (toolchain manager).

2. Scaffold the project:
   ```
   cargo new labwatch --bin
   cd labwatch
   cargo run   # should print "Hello, world!"
   ```

3. Add dependencies to `Cargo.toml` as you need them (don't add all at once):
   - `ssh2` — SSH connections to remote nodes
   - `sysinfo` — local CPU/mem/disk stats
   - `ratatui` + `crossterm` — the terminal dashboard UI
   - `serde` + `serde_json` — for the `--json` output mode
   - `clap` — command-line argument parsing

## Module layout

```
src/
  main.rs        # thin entry point: CLI args, wiring everything together
  node.rs         # the Node struct and its methods
  collector.rs   # SSH polling logic
  dashboard.rs   # ratatui rendering
```

Rust's `mod node;` in `main.rs` works much like splitting files in C/C++.

## Core struct (starting point)

`src/node.rs`:
```rust
#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub cpu: f64,
    pub mem_used: f64,
    pub up: bool,
}

impl Node {
    pub fn status_line(&self) -> String {
        let status = if self.up { "UP" } else { "DOWN" };
        format!("{:<10} {:<6} CPU:{:.1}%  Mem:{:.1}%", self.name, status, self.cpu, self.mem_used)
    }
}
```

`src/main.rs` (first working skeleton, fake data):
```rust
mod node;
use node::Node;

fn main() {
    let fake_nodes = vec![
        Node { name: "laptop-1".into(), cpu: 42.3, mem_used: 61.0, up: true },
        Node { name: "laptop-2".into(), cpu: 12.1, mem_used: 30.5, up: true },
        Node { name: "laptop-3".into(), cpu: 0.0, mem_used: 0.0, up: false },
    ];

    for n in &fake_nodes {
        println!("{}", n.status_line());
    }
}
```

## Concurrent polling (once past fake data)

Use `std::thread` + `std::sync::mpsc` (a channel) rather than shared mutable
state. Each thread owns its own `Node` and sends it across on completion:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn poll_node(name: &str) -> Node {
    thread::sleep(Duration::from_millis(200)); // placeholder for real SSH call
    Node { name: name.to_string(), cpu: 42.3, mem_used: 61.0, up: true }
}

fn poll_all(names: Vec<&str>) -> Vec<Node> {
    let (tx, rx) = mpsc::channel();
    for name in names {
        let tx = tx.clone();
        let name = name.to_string();
        thread::spawn(move || {
            let node = poll_node(&name);
            tx.send(node).unwrap();
        });
    }
    drop(tx);
    rx.iter().collect()
}
```

Why a channel instead of a shared `Vec`/array written from multiple threads:
Rust's compiler will not let two threads mutate the same data without
explicit, checked synchronization. A channel sidesteps that entirely — each
thread produces its own value and hands it off, so there's nothing to race.

## Build order

**Week 1**
- Days 1-2: Skeleton + fake data (above) — prove toolchain and structure work.
- Days 3-4: Real local stats — swap fake numbers for `sysinfo` reading your
  own machine.
- Days 5-7: SSH polling, one node — add `ssh2`, get real stats from exactly
  one other machine. Budget the most time here; it's usually the hardest step.

**Week 2**
- Days 1-2: Scale to all nodes concurrently, using the channel pattern above.
- Days 3-4: Terminal dashboard — replace `println!` with a live, auto-
  refreshing `ratatui` table, color-coded by health thresholds.
- Days 5-6: CLI flags + JSON mode — add `clap` for config path/refresh
  interval, and a `--json`/`--once` flag for scriptable output.
- Day 7: Polish — README against your real home-lab topology, a demo GIF,
  tag `v1.0`.

## Notes

- Started 9/20/2026; targeting completion before HackRPI 2026 (Nov 7-8) eats
  a weekend.
- Debug one layer at a time: fake data → real local data → one remote node →
  all nodes → dashboard → flags. Each step should work end-to-end before
  adding the next.