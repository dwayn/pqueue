# PQueue Server Example

Reference implementation of a TCP server using `concurrent-pqueue`. Demonstrates production-ready patterns for networked priority queue services.

## Architecture

- **Async TCP server** using Tokio
- **Per-connection tasks** with shared queue access
- **Line-based protocol** with CRLF termination
- **Structured command parsing** with error handling

## Protocol

```
UPDATE <id> <score>    # Add/update item priority (additive)
NEXT                   # Pop highest priority item
PEEK                   # View highest priority item  
SCORE <id>             # Get current item priority
INFO                   # Server statistics
HELP                   # Command reference
```

**Response format:**
- Success: `+<result>\r\n`
- Error: `-<message>\r\n`
- Empty queue: `+-1\r\n`

## Usage

```bash
# Start server
cargo run --bin pqueue_server -- --host 0.0.0.0 --port 8002 --debug

# Options
--host <HOST>     # Bind address (default: 0.0.0.0)
--port <PORT>     # Port number (default: 8002)  
--debug           # Enable connection logging
```

## Implementation Details

**Connection handling:**
```rust
let pqueue = Arc::new(PQueue::<String>::new());
let pqueue_clone = pqueue.clone(); // Shared access

tokio::spawn(async move {
    handle_connection(socket, pqueue_clone, debug).await;
});
```

**Command processing:**
- Parse commands with `Command::from()`
- Process with shared queue reference  
- Return structured `Response` enum
- Handle client disconnections gracefully

## Key Patterns

- **Shared state**: `Arc<PQueue<String>>` cloned per connection
- **Async I/O**: Tokio streams with buffered reading
- **Error boundaries**: Graceful handling of malformed commands
- **Resource cleanup**: Automatic cleanup on client disconnect

This example demonstrates real-world usage patterns for integrating `concurrent-pqueue` into network services.