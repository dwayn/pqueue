use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::hash::Hash;
use chrono::{NaiveDateTime, Duration, Utc};

// Priority queue wrapper with internal synchronization using Arc and Mutex for thread safety
// You can clone this and pass it to multiple threads to share the same internal queue. Cloning
// will not copy the data, but instead, each cloned instance will point to the same internal queue.
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
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for PQueue<T>
where
    T: Eq + Hash + Clone,
{
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone()
        }
    }
}

impl<T> PQueue<T>
where
    T: Eq + Hash + Clone,
{
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
            }))
        }
    }

    pub fn update(&self, item: T, new_score: i64) {
        let mut queue = self.queue.lock().unwrap();
        queue.update(Arc::new(item), new_score);
    }

    pub fn peek(&self) -> Option<T> {
        let queue = self.queue.lock().unwrap();
        queue.peek().map(|arc_item| (*arc_item).clone())
    }

    pub fn next(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        queue.next().map(|arc_item| Arc::try_unwrap(arc_item).unwrap_or_else(|arc| (*arc).clone()))
    }

    pub fn score(&self, item: &T) -> Option<i64> {
        let queue = self.queue.lock().unwrap();
        queue.score(&Arc::new(item.clone()))
    }

    pub fn stats(&self) -> PQueueStats {
        let queue = self.queue.lock().unwrap();
        queue.stats.clone().into()
    }
}

/// Statistics for the priority queue, returned by the `stats` method
///
/// uptime: The time since the priority queue was instantiated
/// version: The version of the priority queue lib
/// updates: The count of update calls made to the queue since it was started
/// items: The count of items currently in the queue
/// pools: The count of separate score pools in the queue (a pool is just a set of items with the same score)
#[derive(Clone, Debug)]
pub struct PQueueStats {
    pub uptime: Duration,
    pub version: String,
    pub updates: i64,
    pub items: i64,
    pub pools: i64
}

impl From<PQueueStatsTracker> for PQueueStats {
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

// Statistics tracker for the priority queue
#[derive(Clone, Debug)]
struct PQueueStatsTracker {
    start_time: NaiveDateTime,
    updates: i64,
    items: i64,
    pools: i64,
}

// The core priority queue structure

struct PriorityQueue<T>
where
    T: Eq + Hash,
{
    scores: BTreeMap<i64, VecDeque<Arc<T>>>,
    items: HashMap<Arc<T>, i64>,
    stats: PQueueStatsTracker,
}

impl<T> PriorityQueue<T>
where
    T: Eq + Hash + Clone,
{
    pub fn update(&mut self, item: Arc<T>, new_score: i64) {
        let mut new_score = new_score;
        self.stats.updates += 1;
        if let Some(&current_score) = self.items.get(&item) {
            self.remove_item(&item, current_score);
            new_score += current_score;
        } else {
            self.stats.items += 1;
        }

        self.items.insert(item.clone(), new_score);
        if !self.scores.contains_key(&new_score) {
            self.stats.pools += 1;
        }
        self.scores.entry(new_score).or_default().push_back(item);
    }

    pub fn peek(&self) -> Option<Arc<T>> {
        self.scores.iter().next_back().and_then(|(_, items)| items.iter().next().cloned())
    }

    pub fn next(&mut self) -> Option<Arc<T>> {
        if let Some((&score, items)) = self.scores.iter_mut().next_back() {
            let item = items.pop_front();
            if let Some(item) = item {
                if items.is_empty() {
                    self.scores.remove(&score);
                    self.stats.pools -= 1;
                }
                self.items.remove(&item);
                self.stats.items -= 1;
                Some(item)
            } else {
                self.scores.remove(&score);
                self.stats.pools -= 1;
                None
            }
        } else {
            None
        }
    }

    pub fn score(&self, item: &Arc<T>) -> Option<i64> {
        self.items.get(item).cloned()
    }

    fn remove_item(&mut self, item: &Arc<T>, score: i64) {
        if let Some(items) = self.scores.get_mut(&score) {
            items.retain(|i| i != item);
            if items.is_empty() {
                self.scores.remove(&score);
                self.stats.pools -= 1;
            }
        }
    }
}

pub trait PQueueOperations<T> {
    fn new() -> Self;
    fn update(&self, item: T, new_score: i64);
    fn peek(&self) -> Option<T>;
    fn next(&self) -> Option<T>;
    fn score(&self, item: &T) -> Option<i64>;
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
        queue.update("item1".to_string(), 10);
        queue.update("item1".to_string(), 20);
        assert_eq!(queue.score(&"item1".to_string()), Some(30));
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