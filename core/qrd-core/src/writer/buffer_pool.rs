//! Reusable buffer pool for memory efficiency
//!
//! Provides arena-like memory reuse to avoid repeated allocations
//! during streaming write operations.

use std::sync::Mutex;

/// Configuration for buffer pool behavior
#[derive(Debug, Clone, Copy)]
pub struct BufferPoolConfig {
    /// Initial capacity of each buffer (bytes)
    pub initial_capacity: usize,
    /// Maximum buffers to cache
    pub max_cached_buffers: usize,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        BufferPoolConfig {
            initial_capacity: 65536, // 64 KB default
            max_cached_buffers: 16,
        }
    }
}

/// Manages reusable byte buffers
pub struct BufferPool {
    config: BufferPoolConfig,
    caches: [Mutex<Vec<Vec<u8>>>; 4],
}

impl BufferPool {
    /// Create new buffer pool with default config
    pub fn new() -> Self {
        Self::with_config(BufferPoolConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: BufferPoolConfig) -> Self {
        // Use Default trait to initialize array of Mutexes
        let caches: [Mutex<Vec<Vec<u8>>>; 4] = Default::default();
        BufferPool { config, caches }
    }

    /// Get a buffer by size tier
    ///
    /// Size tiers:
    /// - Tier 0: Small (<64KB)
    /// - Tier 1: Medium (64KB-256KB)
    /// - Tier 2: Large (256KB-1MB)
    /// - Tier 3: XL (>1MB)
    pub fn acquire(&self, min_capacity: usize) -> Vec<u8> {
        let tier = self.tier_for_size(min_capacity);

        if let Ok(mut cache) = self.caches[tier].lock() {
            if let Some(mut buf) = cache.pop() {
                buf.clear();
                // Ensure sufficient capacity
                if buf.capacity() >= min_capacity {
                    return buf;
                }
                // If cached buffer too small, grow it
                buf.reserve(min_capacity - buf.capacity());
                return buf;
            }
        }

        // No cached buffer available, allocate new one
        Vec::with_capacity(min_capacity)
    }

    /// Return buffer to pool for reuse
    pub fn release(&self, buffer: Vec<u8>) {
        let tier = self.tier_for_size(buffer.capacity());

        if let Ok(mut cache) = self.caches[tier].lock() {
            if cache.len() < self.config.max_cached_buffers {
                cache.push(buffer);
            }
            // else: discard oversized cache entry
        }
    }

    /// Clear all cached buffers
    pub fn clear(&self) {
        for cache in self.caches.iter() {
            if let Ok(mut c) = cache.lock() {
                c.clear();
            }
        }
    }

    /// Get approximate pool statistics
    pub fn stats(&self) -> (usize, usize) {
        let mut total_buffers = 0;
        let mut total_capacity = 0;

        for cache in self.caches.iter() {
            if let Ok(c) = cache.lock() {
                total_buffers += c.len();
                total_capacity += c.iter().map(|b| b.capacity()).sum::<usize>();
            }
        }

        (total_buffers, total_capacity)
    }

    /// Determine size tier for buffer
    fn tier_for_size(&self, size: usize) -> usize {
        match size {
            0..=65535 => 0,        // <64KB
            65536..=262143 => 1,   // 64KB-256KB
            262144..=1048575 => 2, // 256KB-1MB
            _ => 3,                // >1MB
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_acquire_release() {
        let pool = BufferPool::new();
        let buf1 = pool.acquire(1024);
        assert!(buf1.capacity() >= 1024);

        pool.release(buf1);
        let (count, cap) = pool.stats();
        assert_eq!(count, 1);
        assert!(cap >= 1024);
    }

    #[test]
    fn test_buffer_reuse() {
        let pool = BufferPool::new();
        let buf1 = pool.acquire(1024);
        let original_ptr = buf1.as_ptr();
        pool.release(buf1);

        let buf2 = pool.acquire(512);
        // Should reuse if capacity sufficient
        if buf2.capacity() >= 1024 {
            assert_eq!(buf2.as_ptr(), original_ptr);
        }
    }

    #[test]
    fn test_tier_for_size() {
        let pool = BufferPool::new();
        assert_eq!(pool.tier_for_size(1000), 0);
        assert_eq!(pool.tier_for_size(100000), 1);
        assert_eq!(pool.tier_for_size(500000), 2);
        assert_eq!(pool.tier_for_size(2000000), 3);
    }

    #[test]
    fn test_cache_limit() {
        let config = BufferPoolConfig {
            initial_capacity: 1024,
            max_cached_buffers: 2,
        };
        let pool = BufferPool::with_config(config);

        // Add 3 buffers
        pool.release(Vec::with_capacity(1024));
        pool.release(Vec::with_capacity(1024));
        pool.release(Vec::with_capacity(1024));

        let (count, _) = pool.stats();
        assert_eq!(count, 2); // Only 2 retained
    }
}
