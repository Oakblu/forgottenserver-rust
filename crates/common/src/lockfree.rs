//! Migrated from forgottenserver/src/lockfree.h
//!
//! The C++ `lockfree.h` provides two things built on Boost.Lockfree:
//!
//! 1. `LockfreeFreeList<TSize, Capacity>` — a per-type-size singleton
//!    lock-free stack of raw `void*` pointers (memory blocks).
//! 2. `LockfreePoolingAllocator<T, Capacity>` — an `std::allocator`-
//!    compatible allocator that recycles `T`-sized blocks through the
//!    free-list instead of always calling `operator new/delete`.
//!
//! In Rust we don't need a custom allocator (the global allocator is
//! efficient), but we do need the equivalent **object-pool** abstraction
//! for two reasons:
//!
//! - `OutputMessage` objects are allocated/deallocated at high frequency
//!   on the network hot path and the C++ code explicitly recycles them.
//! - Code that migrates `outputmessage.rs` will call `LockfreePool::get()`
//!   to obtain a recycled object and drop it back into the pool when done.
//!
//! ## Design
//!
//! We provide two public types:
//!
//! * `LockfreePool<T, const CAP: usize>` — a **bounded** LIFO free-list
//!   (lock-free stack) that holds at most `CAP` idle items.  The internal
//!   stack uses `AtomicPtr`-based CAS, which is the Rust equivalent of
//!   `boost::lockfree::stack`.
//!
//! * `PooledObject<T, const CAP: usize>` — an RAII guard returned by
//!   `LockfreePool::acquire()`.  When dropped it pushes the inner value
//!   back into the pool (if the pool has space); otherwise the value is
//!   simply dropped.
//!
//! ## SPSC Ring Buffer (bonus)
//!
//! The prompt also requests a single-producer / single-consumer ring buffer.
//! Although `lockfree.h` itself only contains the allocator, a lock-free
//! SPSC queue is the other classic lock-free primitive and is commonly
//! bundled together.  We implement `LockfreeRingBuffer<T>` here as well.

#![allow(dead_code)]

use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;

// ============================================================================
// LockfreePool — bounded lock-free object pool
// ============================================================================

/// Node in the free-list stack.
struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

/// A bounded, lock-free object pool.
///
/// * `T`   — the type of pooled objects.
/// * `CAP` — maximum number of idle objects stored in the pool at any time.
///
/// This mirrors the C++ `LockfreeFreeList<sizeof(T), Capacity>` singleton,
/// but is an instance-based (non-singleton) Rust struct for testability.
///
/// # Safety
///
/// The pool uses raw pointer manipulation internally.  The public API is safe.
pub struct LockfreePool<T, const CAP: usize> {
    head: AtomicPtr<Node<T>>,
    count: AtomicUsize,
}

// SAFETY: The pool never gives out aliased mutable references; ownership is
// transferred atomically through the CAS-based stack.
unsafe impl<T: Send, const CAP: usize> Send for LockfreePool<T, CAP> {}
unsafe impl<T: Send, const CAP: usize> Sync for LockfreePool<T, CAP> {}

impl<T, const CAP: usize> LockfreePool<T, CAP> {
    /// Create an empty pool.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            count: AtomicUsize::new(0),
        }
    }

    /// Push an object back onto the free-list.
    ///
    /// Returns `true` if the object was stored, `false` if the pool was
    /// already at capacity and the object was dropped.
    /// Mirrors `bounded_push` in the C++ code.
    pub fn release(&self, value: T) -> bool {
        // Check capacity with acquire to observe concurrent releases.
        if self.count.load(Ordering::Acquire) >= CAP {
            // Pool full — let the value drop.
            return false;
        }

        let node = Box::into_raw(Box::new(Node {
            value,
            next: ptr::null_mut(),
        }));

        loop {
            let old_head = self.head.load(Ordering::Acquire);
            // SAFETY: node is a freshly allocated, non-null pointer.
            unsafe { (*node).next = old_head };

            match self.head.compare_exchange_weak(
                old_head,
                node,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.count.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
                Err(_) => {
                    // Retry; the head changed between load and CAS.
                    continue;
                }
            }
        }
    }

    /// Pop an object from the free-list, returning `None` if empty.
    /// Mirrors the C++ `pop` call.
    pub fn acquire(&self) -> Option<T> {
        loop {
            let old_head = self.head.load(Ordering::Acquire);
            if old_head.is_null() {
                return None;
            }
            // SAFETY: old_head is non-null and was stored by `release()`.
            let next = unsafe { (*old_head).next };
            match self.head.compare_exchange_weak(
                old_head,
                next,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.count.fetch_sub(1, Ordering::Relaxed);
                    // SAFETY: we own this node now — no other thread will touch it.
                    let node = unsafe { Box::from_raw(old_head) };
                    return Some(node.value);
                }
                Err(_) => continue,
            }
        }
    }

    /// Number of objects currently sitting idle in the pool.
    pub fn idle_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

impl<T, const CAP: usize> Default for LockfreePool<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> Drop for LockfreePool<T, CAP> {
    fn drop(&mut self) {
        // Drain all remaining nodes.
        while self.acquire().is_some() {}
    }
}

// ============================================================================
// PooledObject — RAII guard that auto-returns to the pool
// ============================================================================

/// An RAII guard wrapping a `T` that was acquired from a `LockfreePool`.
///
/// When this value is dropped it is automatically returned to the pool if
/// capacity allows, otherwise it is dropped normally.
pub struct PooledObject<T, const CAP: usize> {
    inner: Option<T>,
    pool: Arc<LockfreePool<T, CAP>>,
}

impl<T, const CAP: usize> PooledObject<T, CAP> {
    fn new(value: T, pool: Arc<LockfreePool<T, CAP>>) -> Self {
        Self {
            inner: Some(value),
            pool,
        }
    }
}

impl<T, const CAP: usize> std::ops::Deref for PooledObject<T, CAP> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }
}

impl<T, const CAP: usize> std::ops::DerefMut for PooledObject<T, CAP> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }
}

impl<T, const CAP: usize> Drop for PooledObject<T, CAP> {
    fn drop(&mut self) {
        if let Some(value) = self.inner.take() {
            self.pool.release(value);
        }
    }
}

// ============================================================================
// SharedLockfreePool — Arc wrapper for convenient shared-ownership pools
// ============================================================================

/// A shareable (cloneable) handle to a `LockfreePool`.
///
/// This is the idiomatic way to share a pool across threads, matching the
/// C++ pattern of using a singleton `LockfreeFreeList::get()`.
#[derive(Clone)]
pub struct SharedLockfreePool<T, const CAP: usize>(pub Arc<LockfreePool<T, CAP>>);

impl<T, const CAP: usize> SharedLockfreePool<T, CAP> {
    pub fn new() -> Self {
        Self(Arc::new(LockfreePool::new()))
    }

    /// Acquire an idle object (like `LockfreeFreeList::pop`).
    pub fn acquire(&self) -> Option<T> {
        self.0.acquire()
    }

    /// Return an object to the pool (like `bounded_push`).
    pub fn release(&self, value: T) -> bool {
        self.0.release(value)
    }

    /// Acquire as a `PooledObject` RAII guard.
    pub fn acquire_pooled(&self, value: T) -> PooledObject<T, CAP> {
        PooledObject::new(value, Arc::clone(&self.0))
    }
}

impl<T, const CAP: usize> Default for SharedLockfreePool<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// LockfreeRingBuffer — SPSC ring buffer
// ============================================================================

/// A single-producer single-consumer lock-free ring buffer (FIFO queue).
///
/// Capacity is rounded up to the next power of two internally for efficient
/// index masking.
///
/// # Thread safety
///
/// Only safe for **one** producer thread and **one** consumer thread
/// concurrently.  Multi-producer or multi-consumer use is not supported.
pub struct LockfreeRingBuffer<T> {
    buf: Vec<std::cell::UnsafeCell<std::mem::MaybeUninit<T>>>,
    mask: usize,
    head: AtomicUsize, // consumer reads from head
    tail: AtomicUsize, // producer writes to tail
}

// SAFETY: we guarantee exactly one producer and one consumer.
unsafe impl<T: Send> Send for LockfreeRingBuffer<T> {}
unsafe impl<T: Send> Sync for LockfreeRingBuffer<T> {}

impl<T> LockfreeRingBuffer<T> {
    /// Create a new ring buffer with at least `capacity` slots.
    ///
    /// The actual capacity is rounded up to the next power of two (minimum 2).
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two().max(2);
        let buf = (0..cap)
            .map(|_| std::cell::UnsafeCell::new(std::mem::MaybeUninit::uninit()))
            .collect();
        Self {
            buf,
            mask: cap - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Number of slots (always a power of two).
    pub fn capacity(&self) -> usize {
        self.mask + 1
    }

    /// Push `value` into the ring buffer.
    ///
    /// Returns `false` if the buffer is full and the value was **not** stored.
    pub fn push(&self, value: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        // Full when the number of queued items equals the capacity.
        if tail.wrapping_sub(head) >= self.capacity() {
            return false;
        }

        // SAFETY: tail is only modified by the producer (this thread).
        unsafe {
            let slot = &mut *self.buf[tail & self.mask].get();
            slot.write(value);
        }

        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        true
    }

    /// Pop the next item from the ring buffer.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        // Empty if head == tail
        if head == self.tail.load(Ordering::Acquire) {
            return None;
        }

        // SAFETY: head is only modified by the consumer (this thread).
        let value = unsafe {
            let slot = &*self.buf[head & self.mask].get();
            slot.assume_init_read()
        };

        self.head.store(head.wrapping_add(1), Ordering::Release);
        Some(value)
    }

    /// Returns `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire)
    }

    /// Returns the number of items currently in the buffer.
    pub fn len(&self) -> usize {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        tail.wrapping_sub(head)
    }
}

impl<T> Drop for LockfreeRingBuffer<T> {
    fn drop(&mut self) {
        // Drain remaining items so their destructors run.
        while self.pop().is_some() {}
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    // -----------------------------------------------------------------------
    // LockfreePool tests
    // -----------------------------------------------------------------------

    #[test]
    fn pool_empty_acquire_returns_none() {
        let pool: LockfreePool<u32, 8> = LockfreePool::new();
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn pool_release_then_acquire_roundtrips() {
        let pool: LockfreePool<String, 8> = LockfreePool::new();
        pool.release("hello".to_string());
        let v = pool.acquire().expect("should have one item");
        assert_eq!(v, "hello");
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn pool_respects_capacity() {
        let pool: LockfreePool<u32, 2> = LockfreePool::new();
        assert!(pool.release(1));
        assert!(pool.release(2));
        // Third push must fail (pool full)
        assert!(!pool.release(3));
        assert_eq!(pool.idle_count(), 2);
    }

    #[test]
    fn pool_idle_count_tracks_correctly() {
        let pool: LockfreePool<i32, 10> = LockfreePool::new();
        assert_eq!(pool.idle_count(), 0);
        pool.release(42);
        assert_eq!(pool.idle_count(), 1);
        pool.acquire();
        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn pool_shared_across_threads() {
        let pool = Arc::new(LockfreePool::<u32, 1024>::new());
        let pool2 = Arc::clone(&pool);

        let producer = thread::spawn(move || {
            for i in 0..100u32 {
                pool2.release(i);
            }
        });

        producer.join().unwrap();

        let mut count = 0usize;
        while pool.acquire().is_some() {
            count += 1;
        }
        // At most 100 items could have fit (CAP=1024 > 100, so all should fit).
        assert_eq!(count, 100);
    }

    // -----------------------------------------------------------------------
    // LockfreeRingBuffer tests
    // -----------------------------------------------------------------------

    #[test]
    fn ring_push_when_full_returns_false() {
        let rb = LockfreeRingBuffer::<u32>::new(4); // actual cap = 4
                                                    // Fill it up
        for i in 0..4 {
            assert!(rb.push(i), "push {i} should succeed");
        }
        assert!(!rb.push(99), "push to full buffer should return false");
    }

    #[test]
    fn ring_pop_when_empty_returns_none() {
        let rb = LockfreeRingBuffer::<u32>::new(4);
        assert!(rb.pop().is_none());
    }

    #[test]
    fn ring_fifo_ordering() {
        let rb = LockfreeRingBuffer::<u32>::new(128);
        for i in 0..100u32 {
            assert!(rb.push(i));
        }
        for i in 0..100u32 {
            assert_eq!(rb.pop(), Some(i));
        }
        assert!(rb.pop().is_none());
    }

    #[test]
    fn ring_capacity_is_power_of_two() {
        assert_eq!(LockfreeRingBuffer::<u8>::new(3).capacity(), 4);
        assert_eq!(LockfreeRingBuffer::<u8>::new(4).capacity(), 4);
        assert_eq!(LockfreeRingBuffer::<u8>::new(5).capacity(), 8);
        assert_eq!(LockfreeRingBuffer::<u8>::new(1).capacity(), 2);
    }

    #[test]
    fn ring_len_and_is_empty() {
        let rb = LockfreeRingBuffer::<u32>::new(8);
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
        rb.push(1);
        rb.push(2);
        assert!(!rb.is_empty());
        assert_eq!(rb.len(), 2);
        rb.pop();
        assert_eq!(rb.len(), 1);
    }

    #[test]
    fn ring_concurrent_spsc_1000_items() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let rb = Arc::new(LockfreeRingBuffer::<u32>::new(1024));
        let rb_producer = Arc::clone(&rb);
        let rb_consumer = Arc::clone(&rb);

        const N: u32 = 1000;

        let producer = thread::spawn(move || {
            let mut sent = 0u32;
            while sent < N {
                if rb_producer.push(sent) {
                    sent += 1;
                } else {
                    thread::yield_now();
                }
            }
        });

        let done = Arc::new(AtomicBool::new(false));
        let done_clone = Arc::clone(&done);

        let consumer = thread::spawn(move || {
            let mut received: Vec<u32> = Vec::with_capacity(N as usize);
            while received.len() < N as usize {
                if let Some(v) = rb_consumer.pop() {
                    received.push(v);
                } else {
                    thread::yield_now();
                }
            }
            done_clone.store(true, Ordering::Relaxed);
            received
        });

        producer.join().unwrap();
        let received = consumer.join().unwrap();

        assert!(done.load(Ordering::Relaxed));
        assert_eq!(received.len(), N as usize);
        // Verify FIFO: items must arrive in order 0..N
        for (i, &v) in received.iter().enumerate() {
            assert_eq!(v, i as u32, "FIFO violation at position {i}");
        }
    }

    #[test]
    fn ring_wrap_around_works() {
        // Use a small buffer and push/pop past the wrap point multiple times.
        let rb = LockfreeRingBuffer::<u32>::new(4); // cap = 4

        for round in 0..5u32 {
            for i in 0..4u32 {
                assert!(rb.push(round * 4 + i));
            }
            for i in 0..4u32 {
                assert_eq!(rb.pop(), Some(round * 4 + i));
            }
        }
        assert!(rb.is_empty());
    }

    // -----------------------------------------------------------------------
    // LockfreePool::Default + drain-with-items-on-drop
    // -----------------------------------------------------------------------

    #[test]
    fn pool_default_constructs_empty_pool() {
        // `Default::default()` must mirror `LockfreePool::new()`.
        let pool: LockfreePool<u32, 4> = Default::default();
        assert_eq!(pool.idle_count(), 0);
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn pool_drop_drains_remaining_nodes() {
        // Drop impl must drain remaining entries (exercises the
        // `while self.acquire().is_some() {}` loop in `Drop`).
        use std::sync::atomic::{AtomicUsize, Ordering as AtomOrder};

        static DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Tracked;
        impl Drop for Tracked {
            fn drop(&mut self) {
                DROPS.fetch_add(1, AtomOrder::SeqCst);
            }
        }

        DROPS.store(0, AtomOrder::SeqCst);
        {
            let pool: LockfreePool<Tracked, 8> = LockfreePool::new();
            pool.release(Tracked);
            pool.release(Tracked);
            pool.release(Tracked);
            // Pool falls out of scope here — Drop must drain all 3.
        }
        assert_eq!(DROPS.load(AtomOrder::SeqCst), 3);
    }

    // -----------------------------------------------------------------------
    // PooledObject RAII guard tests
    // -----------------------------------------------------------------------

    #[test]
    fn pooled_object_deref_exposes_inner_value() {
        let shared: SharedLockfreePool<u32, 4> = SharedLockfreePool::new();
        let guard = shared.acquire_pooled(7u32);
        // Deref
        assert_eq!(*guard, 7);
    }

    #[test]
    fn pooled_object_deref_mut_allows_mutation() {
        let shared: SharedLockfreePool<u32, 4> = SharedLockfreePool::new();
        let mut guard = shared.acquire_pooled(10u32);
        *guard += 5;
        assert_eq!(*guard, 15);
    }

    #[test]
    fn pooled_object_drop_returns_value_to_pool() {
        let shared: SharedLockfreePool<u32, 4> = SharedLockfreePool::new();
        assert_eq!(shared.0.idle_count(), 0);
        {
            let _guard = shared.acquire_pooled(42u32);
            // While the guard is alive, the value is not yet in the pool.
            assert_eq!(shared.0.idle_count(), 0);
        }
        // After drop, the value is back in the pool.
        assert_eq!(shared.0.idle_count(), 1);
        assert_eq!(shared.acquire(), Some(42));
    }

    #[test]
    fn pooled_object_drop_when_pool_full_drops_value() {
        // When the pool is full, the dropped guard must drop the value
        // rather than crashing or growing the pool past CAP.
        use std::sync::atomic::{AtomicUsize, Ordering as AtomOrder};

        static DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Tracked;
        impl Drop for Tracked {
            fn drop(&mut self) {
                DROPS.fetch_add(1, AtomOrder::SeqCst);
            }
        }

        DROPS.store(0, AtomOrder::SeqCst);
        let shared: SharedLockfreePool<Tracked, 1> = SharedLockfreePool::new();
        // Fill the pool to capacity.
        assert!(shared.release(Tracked));
        assert_eq!(shared.0.idle_count(), 1);

        {
            let _guard = shared.acquire_pooled(Tracked);
            // Pool is already at CAP=1, so when guard drops the value
            // cannot be stored and must be dropped instead.
        }
        // Pool count must remain at 1 (the originally released item).
        assert_eq!(shared.0.idle_count(), 1);
        // The guard-owned Tracked must have been dropped.
        assert!(DROPS.load(AtomOrder::SeqCst) >= 1);
    }

    // -----------------------------------------------------------------------
    // SharedLockfreePool tests
    // -----------------------------------------------------------------------

    #[test]
    fn shared_pool_new_starts_empty() {
        let shared: SharedLockfreePool<u32, 8> = SharedLockfreePool::new();
        assert_eq!(shared.0.idle_count(), 0);
        assert!(shared.acquire().is_none());
    }

    #[test]
    fn shared_pool_default_starts_empty() {
        let shared: SharedLockfreePool<u32, 8> = SharedLockfreePool::default();
        assert_eq!(shared.0.idle_count(), 0);
        assert!(shared.acquire().is_none());
    }

    #[test]
    fn shared_pool_release_and_acquire_roundtrip() {
        let shared: SharedLockfreePool<String, 4> = SharedLockfreePool::new();
        assert!(shared.release("alpha".to_string()));
        assert_eq!(shared.0.idle_count(), 1);
        let v = shared.acquire().expect("should have one item");
        assert_eq!(v, "alpha");
        assert!(shared.acquire().is_none());
    }

    #[test]
    fn shared_pool_clone_shares_underlying_pool() {
        // Cloning must alias the same Arc, so releases via one handle
        // are visible via the clone.
        let shared: SharedLockfreePool<u32, 8> = SharedLockfreePool::new();
        let clone = shared.clone();
        assert!(shared.release(123));
        // The clone observes the same item.
        assert_eq!(clone.acquire(), Some(123));
        assert_eq!(shared.0.idle_count(), 0);
    }

    #[test]
    fn shared_pool_release_returns_false_when_full() {
        let shared: SharedLockfreePool<u32, 1> = SharedLockfreePool::new();
        assert!(shared.release(1));
        // Second release exceeds CAP=1.
        assert!(!shared.release(2));
        assert_eq!(shared.0.idle_count(), 1);
    }

    #[test]
    fn shared_pool_acquire_pooled_yields_working_guard() {
        // acquire_pooled must produce a PooledObject whose Deref returns
        // the original value, and whose Drop returns it to the pool.
        let shared: SharedLockfreePool<i64, 4> = SharedLockfreePool::new();
        let mut guard = shared.acquire_pooled(-1i64);
        assert_eq!(*guard, -1);
        *guard = 99;
        drop(guard);
        // After dropping the guard, the (mutated) value sits in the pool.
        assert_eq!(shared.acquire(), Some(99));
    }
}
