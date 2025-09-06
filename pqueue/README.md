# concurrent-pqueue

Core library implementing a thread-safe priority queue with dynamic priority updates.

## Architecture

**Dual-index design** for optimal performance:
- `BTreeMap<i64, VecDeque<Arc<T>>>` - Ordered priority access (highest first)
- `HashMap<Arc<T>, i64>` - O(1) item-to-score lookups
- `Arc<Mutex<PriorityQueue<T>>>` - Internal thread-safe wrapper

**Key characteristics:**
- Additive scoring: Updates add to existing scores
- FIFO ordering: Equal priority items processed in insertion order
- Zero-copy memory model: Queue only maintains a single copy of an object stored in the queue across all internal representations
- Zero-copy queue cloning: For sharing queue across threads
- Built-in statistics tracking

## Usage

```rust
use concurrent_pqueue::PQueue;

let queue = PQueue::<String>::new();

// Basic operations
queue.update("item1".to_string(), 10);
let item = queue.next();           // Remove highest priority
let item = queue.peek();           // View without removing
let score = queue.score(&item);    // Get current priority
let stats = queue.stats();         // Queue statistics
```

## Performance

- **Insert/Update**: O(log n)
- **Peek**: O(log n)
- **Next (pop)**: O(log n)
- **Score lookup**: O(1)
- **Space**: O(n)

## Thread Safety

Thread-safe by design. Clone instances share the same internal queue:

```rust
let queue = PQueue::<String>::new();
let queue_clone = queue.clone(); // Same underlying data

// Use across threads
tokio::spawn(async move {
    queue_clone.update("task".to_string(), 100);
});
```

## Requirements

- **Type constraints**: `T: Eq + Hash + Clone`
- **Rust version**: 1.70+
- **Dependencies**: `chrono` for timestamps