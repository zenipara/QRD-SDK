//! Memory profiling utilities for QRD operations
//!
//! Tracks memory usage, allocation patterns, and provides insights
//! into memory efficiency of encoding, compression, and I/O operations.

use crate::error::Result;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Global memory profiler allocator
#[global_allocator]
static PROFILER: MemoryProfiler = MemoryProfiler::new();

/// Memory profiling statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Total bytes currently allocated
    pub current_bytes: usize,
    /// Peak bytes allocated during profiling
    pub peak_bytes: usize,
    /// Total allocation operations
    pub total_allocations: usize,
    /// Total deallocation operations
    pub total_deallocations: usize,
    /// Total bytes allocated (cumulative)
    pub total_bytes_allocated: usize,
    /// Total bytes deallocated (cumulative)
    pub total_bytes_deallocated: usize,
}

/// Memory profiler allocator wrapper
pub struct MemoryProfiler {
    allocator: System,
    stats: Mutex<MemoryStats>,
}

impl MemoryProfiler {
    /// Create new memory profiler
    pub const fn new() -> Self {
        MemoryProfiler {
            allocator: System,
            stats: Mutex::new(MemoryStats::default()),
        }
    }

    /// Get current memory statistics
    pub fn stats() -> MemoryStats {
        PROFILER.stats.lock().unwrap().clone()
    }

    /// Reset memory statistics
    pub fn reset_stats() {
        let mut stats = PROFILER.stats.lock().unwrap();
        *stats = MemoryStats::default();
    }

    /// Profile a closure and return its result plus memory stats
    pub fn profile<T, F: FnOnce() -> T>(f: F) -> (T, MemoryStats) {
        // Reset stats before profiling
        Self::reset_stats();

        // Execute the closure
        let result = f();

        // Get final stats
        let stats = Self::stats();

        (result, stats)
    }
}

unsafe impl GlobalAlloc for MemoryProfiler {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.allocator.alloc(layout);

        if !ptr.is_null() {
            let mut stats = self.stats.lock().unwrap();
            stats.total_allocations += 1;
            stats.total_bytes_allocated += layout.size();
            stats.current_bytes += layout.size();

            if stats.current_bytes > stats.peak_bytes {
                stats.peak_bytes = stats.current_bytes;
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout);

        let mut stats = self.stats.lock().unwrap();
        stats.total_deallocations += 1;
        stats.total_bytes_deallocated += layout.size();
        stats.current_bytes = stats.current_bytes.saturating_sub(layout.size());
    }
}

/// Memory profiling scope guard
pub struct MemoryProfileScope {
    start_stats: MemoryStats,
    label: String,
}

impl MemoryProfileScope {
    /// Start profiling a scope
    pub fn new(label: impl Into<String>) -> Self {
        let start_stats = MemoryProfiler::stats();
        MemoryProfileScope {
            start_stats,
            label: label.into(),
        }
    }

    /// Get memory delta since scope start
    pub fn memory_delta(&self) -> MemoryStats {
        let current = MemoryProfiler::stats();

        MemoryStats {
            current_bytes: current.current_bytes.saturating_sub(self.start_stats.current_bytes),
            peak_bytes: current.peak_bytes.saturating_sub(self.start_stats.peak_bytes),
            total_allocations: current.total_allocations.saturating_sub(self.start_stats.total_allocations),
            total_deallocations: current.total_deallocations.saturating_sub(self.start_stats.total_deallocations),
            total_bytes_allocated: current.total_bytes_allocated.saturating_sub(self.start_stats.total_bytes_allocated),
            total_bytes_deallocated: current.total_bytes_deallocated.saturating_sub(self.start_stats.total_bytes_deallocated),
        }
    }
}

impl Drop for MemoryProfileScope {
    fn drop(&mut self) {
        let delta = self.memory_delta();
        println!("Memory profile '{}' - Allocated: {} bytes, Peak: {} bytes, Operations: {} alloc / {} dealloc",
                 self.label,
                 delta.total_bytes_allocated,
                 delta.peak_bytes,
                 delta.total_allocations,
                 delta.total_deallocations);
    }
}

/// Profile memory usage of a QRD write operation
pub fn profile_writer_memory_usage<F>(label: &str, operation: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let _scope = MemoryProfileScope::new(label);
    operation()?;
    Ok(())
}

/// Profile memory usage of a QRD read operation
pub fn profile_reader_memory_usage<F, T>(label: &str, operation: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let _scope = MemoryProfileScope::new(label);
    operation()
}

/// Get human-readable memory size string
pub fn format_memory_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldType, Nullability, SchemaBuilder};
    use crate::writer::FileWriter;
    use tempfile::NamedTempFile;

    #[test]
    fn test_memory_profiling_basic() {
        // Reset stats
        MemoryProfiler::reset_stats();

        // Allocate some memory
        let _vec: Vec<u8> = vec![0; 1024];

        let stats = MemoryProfiler::stats();
        assert!(stats.total_bytes_allocated >= 1024);
        assert!(stats.total_allocations >= 1);
    }

    #[test]
    fn test_memory_profile_scope() {
        let _scope = MemoryProfileScope::new("test_scope");

        let _vec: Vec<u8> = vec![0; 2048];

        // Scope will print stats when dropped
    }

    #[test]
    fn test_profile_writer_operation() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("data", FieldType::Blob, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let result = profile_writer_memory_usage("test_writer", || {
            let mut writer = FileWriter::new(temp.path(), schema.clone())?;

            for i in 0..100 {
                let data = vec![(i % 256) as u8; 100];
                let blob_data = serialize_blob(&data);
                writer.write_row(vec![blob_data])?;
            }

            writer.finish()?;
            Ok(())
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_format_memory_size() {
        assert_eq!(format_memory_size(0), "0 B");
        assert_eq!(format_memory_size(512), "512 B");
        assert_eq!(format_memory_size(1024), "1.0 KB");
        assert_eq!(format_memory_size(1536), "1.5 KB");
        assert_eq!(format_memory_size(1048576), "1.0 MB");
    }

    fn serialize_blob(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&(data.len() as u32).to_le_bytes());
        result.extend_from_slice(data);
        result
    }
}