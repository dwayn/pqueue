# concurrent-pqueue - Rust Priority Queue Library

[![Crates.io](https://img.shields.io/crates/v/concurrent-pqueue.svg)](https://crates.io/crates/concurrent-pqueue)
[![Documentation](https://docs.rs/concurrent-pqueue/badge.svg)](https://docs.rs/concurrent-pqueue)
[![CI](https://github.com/dwayn/pqueue/workflows/CI/badge.svg)](https://github.com/dwayn/pqueue/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE.txt)

A high-performance, thread-safe priority queue implementation in Rust with support for dynamic priority updates. Designed for scenarios where you need to efficiently manage prioritized items with the ability to update priorities after insertion.

## Features

- **Thread-safe**: Built with `Arc<Mutex<T>>` for safe concurrent access across multiple threads
- **Dynamic priority updates**: Update item priorities after insertion with additive scoring
- **Efficient dual-index design**: Uses `BTreeMap` for ordered access and `HashMap` for O(1) lookups
- **Zero-copy cloning**: Clone instances share the same underlying data structure
- **Statistics tracking**: Built-in performance and usage statistics
- **Memory efficient**: Uses `Arc<T>` to avoid unnecessary item cloning
- **FIFO ordering**: Items with equal priority are processed in first-in, first-out order

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
concurrent-pqueue = "0.3.0"
```

### Basic Usage

```rust
use concurrent_pqueue::PQueue;

fn main() {
    let queue = PQueue::<String>::new();

    // Add items with priorities
    queue.update("task_1".to_string(), 10);
    queue.update("task_2".to_string(), 20);
    queue.update("urgent_task".to_string(), 30);

    // Process highest priority item
    if let Some(item) = queue.next() {
        println!("Processing: {}", item); // "urgent_task"
    }

    // Peek without removing
    if let Some(item) = queue.peek() {
        println!("Next item: {}", item); // "task_2"
    }

    // Update priority (additive)
    queue.update("task_1".to_string(), 15); // Now has priority 25

    // Check current priority
    if let Some(score) = queue.score(&"task_1".to_string()) {
        println!("Current priority: {}", score); // 25
    }
}
```

## Advanced Usage

### Custom Types

PQueue works with any type that implements `Eq`, `Hash`, and `Clone`. Here's an example with a custom struct:

```rust
use std::hash::Hash;
use concurrent_pqueue::PQueue;

#[derive(Clone, Debug, Eq)]
pub struct Task {
    id: u64,
    name: String,
    category: String,
}

impl Hash for Task {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state); // Use ID as the unique identifier
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn main() {
    let queue = PQueue::<Task>::new();

    let task = Task {
        id: 1,
        name: "Process data".to_string(),
        category: "computation".to_string(),
    };

    queue.update(task, 100);

    if let Some(next_task) = queue.next() {
        println!("Processing task: {}", next_task.name);
    }
}
```

### Thread Safety

PQueue is designed for concurrent use. Simply clone the queue to share it across threads:

```rust
use concurrent_pqueue::PQueue;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() {
    let queue = PQueue::<String>::new();

    // Producer task
    let producer_queue = queue.clone();
    tokio::spawn(async move {
        for i in 0..10 {
            producer_queue.update(format!("task_{}", i), i * 10);
        }
    });

    // Consumer task
    let consumer_queue = queue.clone();
    tokio::spawn(async move {
        while let Some(item) = consumer_queue.next() {
            println!("Processing: {}", item);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}
```

### Monitoring and Statistics

Track queue performance with built-in statistics:

```rust
use concurrent_pqueue::PQueue;

fn main() {
    let queue = PQueue::<String>::new();

    queue.update("item1".to_string(), 10);
    queue.update("item2".to_string(), 20);
    queue.next();

    let stats = queue.stats();
    println!("Queue Statistics:");
    println!("  Uptime: {} seconds", stats.uptime.num_seconds());
    println!("  Total updates: {}", stats.updates);
    println!("  Items in queue: {}", stats.items);
    println!("  Priority pools: {}", stats.pools);
    println!("  Version: {}", stats.version);
}
```

## API Reference

### Core Methods

- `new() -> PQueue<T>` - Creates a new empty priority queue
- `update(item: T, score: i64) -> (Option<i64>, i64)` - Updates item priority (additive)
- `next() -> Option<T>` - Removes and returns highest priority item
- `peek() -> Option<T>` - Returns highest priority item without removing
- `score(item: &T) -> Option<i64>` - Gets current priority for an item
- `stats() -> PQueueStats` - Returns queue statistics

### Priority System

- **Additive scoring**: When updating an existing item, the new score is added to the current score
- **Highest first**: Items with higher scores are processed first
- **FIFO for ties**: Items with equal priority are processed in insertion order

## Server/Client Implementation

This repository includes a complete TCP server/client implementation demonstrating PQueue usage in a networked environment.

### Running the Server

```bash
cargo run --bin pqueue_server -- --host 0.0.0.0 --port 8002 --debug
```

### Running the Client

```bash
cargo run --bin pqueue_client -- --host localhost --port 8002 --debug
```

### Protocol Commands

```
UPDATE <identifier> <score>  # Updates priority (additive)
NEXT                         # Pops highest priority item
PEEK                         # Views highest priority item
SCORE <identifier>           # Gets current priority
INFO                         # Server statistics
HELP                         # Command help
```

## Implementation Details

### Architecture

- **Dual-index design**: `BTreeMap<i64, VecDeque<Arc<T>>>` for ordered priority access + `HashMap<Arc<T>, i64>` for O(1) lookups
- **Thread safety**: `Arc<Mutex<PriorityQueue<T>>>` wrapper for safe concurrent access
- **Memory efficiency**: Items stored as `Arc<T>` to minimize cloning
- **Score pools**: Items with identical priorities grouped in FIFO queues

### Performance Characteristics

- **Insert/Update**: O(log n) average case
- **Peek**: O(log n) to find highest priority pool, O(1) to access item
- **Next (pop)**: O(log n) average case
- **Score lookup**: O(1) hash map access
- **Space complexity**: O(n) where n is the number of unique items

## Requirements

- **Rust version**: 1.70 or later
- **Dependencies**: `chrono` for timestamp handling

## Testing

Run the test suite:

```bash
cargo test --workspace
```

The library includes comprehensive unit tests covering:
- Basic queue operations (insert, peek, pop)
- Priority updates and additive scoring
- Thread safety scenarios
- Edge cases (empty queue, duplicate items)
- Statistics accuracy

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.
