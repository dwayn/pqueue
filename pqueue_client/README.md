# PQueue Client Example

Interactive TCP client demonstrating connection patterns and protocol usage for `concurrent-pqueue` server integration.

## Features

- **Interactive mode**: REPL with command prompts
- **Non-interactive mode**: Scriptable for automation
- **Async I/O**: Concurrent handling of user input and server responses
- **Auto-detection**: TTY detection for appropriate UI mode

## Usage

```bash
# Interactive client
cargo run --bin pqueue_client -- --host localhost --port 8002

# With debug output
cargo run --bin pqueue_client -- --host localhost --port 8002 --debug

# Scripted usage
echo -e "UPDATE task1 100\nNEXT\nINFO" | cargo run --bin pqueue_client
```

## Implementation Patterns

**Concurrent I/O handling:**
```rust
select! {
    command = stdin.next_line() => {
        // Process user input
    }
    response = reader.next_line() => {
        // Handle server responses
    }
}
```

**Mode detection:**
```rust
let is_interactive = atty::is(atty::Stream::Stdin);
if is_interactive {
    print!("pqueue::{}:{}> ", host, port);
}
```

## Key Features

- **Bidirectional communication**: Simultaneous read/write operations
- **Connection management**: Graceful handling of disconnections
- **Input validation**: Clean command parsing and transmission
- **Error handling**: Network error recovery patterns

## Protocol Examples

```
pqueue::localhost:8002> UPDATE task1 50
+OK
pqueue::localhost:8002> UPDATE task2 100  
+OK
pqueue::localhost:8002> NEXT
+task2
pqueue::localhost:8002> SCORE task1
+50
```

This client serves as a reference for building applications that interact with priority queue services over TCP.