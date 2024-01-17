# Rust Priority Queue library

A threadsafe, heap based, priority queue with the ability to update the priority of items already in the queue.
Internally, it uses a btree for handling managing the prioritization of items, and a hashmap for indexing the items. The
items themselves are all passed by reference using an Arc to avoid needing to clone the items as they are added to the
queue.

# Implementation

PQueue can handle any type `T` that implements `Eq`, `Hash`, and `Clone`, and when an item is added to the queue, PQueue
will take ownership of the item, wrap it in an `Arc<T>` and use these references all throughout the implementation of the
queue. Note that the queue generally will not need to clone an item that is added to the or popped off, but if you `PEEK`
at the first item in the queue, it will clone that item to return it, and when calling `SCORE`, it will clone the item you
pass in to use it to look it up in the item index.

Included in this is a PQueue server and CLI interactive client implementation for a priority queue that just queues
string identifiers with some score. This implements all of the operations that can be done on a priority queue with a
simple interface to serve as a straightforward example of how to use a `PQueue` (or if you just need a standalone
priority queueing daemon for strings, you can just use these binaries as is).

### Example of Defining a Custom Struct to Use in a PQueue
Example custom implementations for Hash and Eq, by implementing PartialEq for custom comparison.
```
use std::hash::Hash;

#[derive(Clone, Debug, Eq)]
pub struct MyType {
    id: i32,
    name: String,
}

// Treats MyType objects as being identified as the same object if their names match
impl Hash for MyType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

// Treats MyType objects as being equal objects if their names match case insensitively
impl PartialEq for MyType {
    fn eq(&self, other: &Self) -> bool {
        self.name.to_lowercase() == other.name.to_lowercase()
    }
}
```

### Sharing a PQueue Across Multiple Threads
When sharing a PQueue for use in multiple threads, you can wrap the PQueue in an Arc and clone the reference to pass to
each of the threads. The internals of the queue have the needed mutexes to ensure thread safety.
```
    let pqueue = Arc::new(PQueue::<String>::new()); // Replace String with your item type

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let pqueue_clone = pqueue.clone();

        tokio::spawn(async move {
            handle_connection(socket, pqueue_clone, debug).await;
        });
    }
```
