use chrono::{Duration, NaiveDateTime, Utc};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// Priority queue wrapper with internal synchronization using Arc and Mutex for thread safety.
///
/// You can clone this and pass it to multiple threads to share the same internal queue. Cloning
/// will not copy the data, but instead, each cloned instance will point to the same internal queue.
pub struct PQueue<T>
where
    T: Eq + Hash + Clone,
{
    queue: Arc<Mutex<PriorityQueue<T>>>,
}

impl<T> Default for PQueue<T>
where
    T: Eq + Hash + Clone,
{
    /// Creates a new empty priority queue using default initialization.
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for PQueue<T>
where
    T: Eq + Hash + Clone,
{
    /// Creates a shallow clone that shares the same internal queue.
    /// Multiple cloned instances will operate on the same underlying data.
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T> PQueue<T>
where
    T: Eq + Hash + Clone,
{
    /// Creates a new empty priority queue with thread-safe `Arc<Mutex<T>>` wrapper.
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(PriorityQueue {
                scores: BTreeMap::new(),
                items: HashMap::new(),
                stats: PQueueStatsTracker {
                    start_time: Utc::now().naive_utc(),
                    updates: 0,
                    items: 0,
                    pools: 0,
                },
            })),
        }
    }

    /// Update the score of an item in the queue or adds it if it doesn't yet
    /// exist.
    ///
    /// Returns a tuple of the old score (`None` if the item didn't yet exist)
    /// and the new score.
    pub fn update(&self, item: T, new_score: i64) -> (Option<i64>, i64) {
        let mut queue = self.queue.lock().unwrap();

        queue.update(Arc::new(item), new_score)
    }

    /// Peek at the highest scoring item in the queue.
    ///
    /// Returns the item with the highest score, or `None` if the queue is
    /// empty.
    pub fn peek(&self) -> Option<T> {
        let queue = self.queue.lock().unwrap();

        queue.peek().map(|arc_item| (*arc_item).clone())
    }

    /// Remove and return the highest scoring item from the queue.
    ///
    /// Returns the item with the highest score, or `None` if the queue is
    /// empty.
    pub fn next(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();

        queue
            .next()
            // Attempt to unwrap Arc, fallback to clone if other references exist
            .map(|arc_item| Arc::try_unwrap(arc_item).unwrap_or_else(|arc| (*arc).clone()))
    }

    /// Get the current score of an item in the queue.
    ///
    /// Returns the score of the item, or `None` if the item doesn't exist in
    /// the queue.
    pub fn score(&self, item: &T) -> Option<i64> {
        let queue = self.queue.lock().unwrap();

        // Create Arc wrapper for lookup (HashMap key consistency)
        queue.score(&Arc::new(item.clone()))
    }

    /// Get the statistics of the priority queue.
    ///
    /// Returns the statistics of the priority queue.
    pub fn stats(&self) -> PQueueStats {
        let queue = self.queue.lock().unwrap();

        queue.stats.clone().into()
    }
}

/// Statistics for the priority queue, returned by the `stats` method.
#[derive(Clone, Debug)]
pub struct PQueueStats {
    /// The time since the priority queue was instantiated
    pub uptime: Duration,
    /// The version of the priority queue lib
    pub version: String,
    /// The count of update calls made to the queue since it was started
    pub updates: i64,
    /// The count of items currently in the queue
    pub items: i64,
    /// The count of separate score pools in the queue (a pool is just a set
    /// of items with the same score)
    pub pools: i64,
}

impl From<PQueueStatsTracker> for PQueueStats {
    /// Converts internal stats tracker to public stats format with computed uptime.
    fn from(value: PQueueStatsTracker) -> Self {
        Self {
            uptime: Utc::now().naive_utc() - value.start_time,
            version: env!("CARGO_PKG_VERSION").to_string(),
            updates: value.updates,
            items: value.items,
            pools: value.pools,
        }
    }
}

/// Statistics tracker for the priority queue
#[derive(Clone, Debug)]
struct PQueueStatsTracker {
    /// The time the priority queue was instantiated
    start_time: NaiveDateTime,
    /// The count of update calls made to the queue since it was started
    updates: i64,
    /// The count of items currently in the queue
    items: i64,
    /// The count of separate score pools in the queue (a pool is just a set
    /// of items with the same score)
    pools: i64,
}

/// The core priority queue structure using a dual-index design:
/// - BTreeMap for ordered access to scores (highest first)
/// - HashMap for O(1) item-to-score lookups
///
/// Items with the same score are stored in a VecDeque for FIFO ordering.
struct PriorityQueue<T>
where
    T: Eq + Hash,
{
    /// Maps scores to queues of items (BTreeMap keeps scores sorted)
    scores: BTreeMap<i64, VecDeque<Arc<T>>>,
    /// Maps items to their current scores for fast lookups
    items: HashMap<Arc<T>, i64>,
    /// Internal statistics tracking
    stats: PQueueStatsTracker,
}

impl<T> PriorityQueue<T>
where
    T: Eq + Hash + Clone,
{
    /// Update the score of an item in the queue or adds it if it doesn't yet
    /// exist.
    ///
    /// Returns a tuple of the old score (`None` if the item didn't yet exist)
    /// and the new score.
    pub fn update(&mut self, item: Arc<T>, new_score: i64) -> (Option<i64>, i64) {
        let mut old_score = None;
        let mut new_score = new_score;

        self.stats.updates += 1;
        if let Some(&current_score) = self.items.get(&item) {
            old_score = Some(current_score);

            self.remove_item(&item, current_score);
            // Additive scoring: new score is added to existing score
            new_score += current_score;
        } else {
            self.stats.items += 1;
        }

        self.items.insert(item.clone(), new_score);
        // Track pool creation: a pool is a set of items with the same score
        if !self.scores.contains_key(&new_score) {
            self.stats.pools += 1;
        }
        self.scores.entry(new_score).or_default().push_back(item);

        (old_score, new_score)
    }

    /// Peek at the highest scoring item in the queue.
    ///
    /// Returns the item with the highest score, or `None` if the queue is
    /// empty.
    pub fn peek(&self) -> Option<Arc<T>> {
        self.scores
            .iter()
            .next_back()
            .and_then(|(_, items)| items.iter().next().cloned())
    }

    /// Remove and return the highest scoring item from the queue.
    ///
    /// Returns the item with the highest score, or `None` if the queue is
    /// empty.
    pub fn next(&mut self) -> Option<Arc<T>> {
        if let Some((&score, items)) = self.scores.iter_mut().next_back() {
            let item = items.pop_front();
            if let Some(item) = item {
                // Clean up empty pools to maintain accurate pool count
                if items.is_empty() {
                    self.scores.remove(&score);
                    self.stats.pools -= 1;
                }
                self.items.remove(&item);
                self.stats.items -= 1;
                Some(item)
            } else {
                // Edge case: empty pool cleanup
                self.scores.remove(&score);
                self.stats.pools -= 1;
                None
            }
        } else {
            None
        }
    }

    /// Get the current score of an item in the queue.
    ///
    /// Returns the score of the item, or `None` if the item doesn't exist in
    /// the queue.
    pub fn score(&self, item: &Arc<T>) -> Option<i64> {
        self.items.get(item).cloned()
    }

    /// Removes an item from a specific score pool and cleans up empty pools.
    /// Used internally when updating existing items to new scores.
    fn remove_item(&mut self, item: &Arc<T>, score: i64) {
        if let Some(items) = self.scores.get_mut(&score) {
            items.retain(|i| i != item);
            // Clean up empty pools to prevent memory leaks and maintain accurate stats
            if items.is_empty() {
                self.scores.remove(&score);
                self.stats.pools -= 1;
            }
        }
    }
}

/// Trait defining the core operations for a priority queue.
/// This abstraction allows for different implementations while maintaining a consistent API.
pub trait PQueueOperations<T> {
    /// Creates a new empty priority queue.
    fn new() -> Self;
    /// Updates an item's score (additive) or adds it if it doesn't exist.
    fn update(&self, item: T, new_score: i64);
    /// Returns the highest-scoring item without removing it.
    fn peek(&self) -> Option<T>;
    /// Removes and returns the highest-scoring item.
    fn next(&self) -> Option<T>;
    /// Gets the current score for a specific item.
    fn score(&self, item: &T) -> Option<i64>;
    /// Returns current queue statistics.
    fn stats(&self) -> PQueueStats;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_and_peek() {
        let queue = PQueue::<String>::new();
        queue.update("item1".to_string(), 10);
        queue.update("item2".to_string(), 20);
        assert_eq!(queue.peek(), Some("item2".to_string()));
    }

    #[test]
    fn test_next() {
        let queue = PQueue::<String>::new();
        queue.update("item1".to_string(), 10);
        queue.update("item2".to_string(), 20);
        assert_eq!(queue.next(), Some("item2".to_string()));
        assert_eq!(queue.peek(), Some("item1".to_string()));
    }

    #[test]
    fn test_update_existing_item() {
        let queue = PQueue::<String>::new();
        let (old_score, new_score) = queue.update("item1".to_string(), 10);
        assert_eq!(old_score, None);
        assert_eq!(new_score, 10);

        let (old_score, new_score) = queue.update("item1".to_string(), 20);
        assert_eq!(old_score, Some(10));
        assert_eq!(new_score, 30);

        assert_eq!(queue.score(&"item1".to_string()), Some(new_score));
    }

    #[test]
    fn test_next_on_empty() {
        let queue = PQueue::<String>::new();
        assert_eq!(queue.next(), None);
    }

    #[test]
    fn test_score_retrieval() {
        let queue = PQueue::<String>::new();
        queue.update("item1".to_string(), 10);
        queue.update("item2".to_string(), 20);
        assert_eq!(queue.score(&"item1".to_string()), Some(10));
        assert_eq!(queue.score(&"item2".to_string()), Some(20));
    }

    #[test]
    fn test_score_after_update() {
        let queue = PQueue::<String>::new();
        queue.update("item1".to_string(), 10);
        queue.update("item1".to_string(), 20); // Updating the same item
        assert_eq!(queue.score(&"item1".to_string()), Some(30)); // Expect the score to be cumulative
    }

    #[test]
    fn test_stats_after_operations() {
        let queue = PQueue::<String>::new();
        queue.update("item1".to_string(), 10);
        queue.update("item2".to_string(), 20);
        queue.next();
        let stats = queue.stats();
        assert_eq!(stats.updates, 2);
        assert_eq!(stats.items, 1); // One item should have been removed
        assert_eq!(stats.pools, 1); // Pools count after one removal
    }

    #[test]
    fn test_removal_of_items() {
        let queue = PQueue::<String>::new();
        queue.update("item1".to_string(), 10);
        queue.update("item2".to_string(), 20);
        queue.next(); // This should remove "item2"
        assert_eq!(queue.score(&"item2".to_string()), None); // "item2" should not be in the queue
    }

    #[test]
    fn test_complex_scenario() {
        let queue = PQueue::<String>::new();
        let queue2 = queue.clone();
        queue.update("item1".to_string(), 10);
        queue.update("item2".to_string(), 15);
        // ensure that queue and it clone share the same internal queue by adding an item to queue2
        // and checking if it comes back when we peek from queue
        queue2.update("item3".to_string(), 22);
        queue2.update("item4".to_string(), 15);
        queue.update("item1".to_string(), 6); // Increment item1's score
        assert_eq!(queue.peek(), Some("item3".to_string())); // "item3" should have the highest score
        queue.next(); // Remove "item3"
        assert_eq!(queue.peek(), Some("item1".to_string())); // "item1" should have the highest score now
        queue.next(); // remove "item1"
        assert_eq!(queue.peek(), Some("item2".to_string())); // Now "item2" should be at the top since it got score 15 before item4 did
        queue.next(); // remove "item2"
        assert_eq!(queue.peek(), Some("item4".to_string())); // Now "item4" is at the front of the queue
    }
}
