//! Определение CPU, RAM, GPU.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::LocalProviderError;

static DETECTED_THREADS: AtomicUsize = AtomicUsize::new(0);

/// Количество логических ядер CPU.
pub fn cpu_cores() -> Result<usize, LocalProviderError> {
    let cached = DETECTED_THREADS.load(Ordering::Relaxed);
    if cached > 0 {
        return Ok(cached);
    }
    let count = std::thread::available_parallelism()
        .map_err(|e| LocalProviderError::HardwareDetectionFailed(e.to_string()))?
        .get();
    DETECTED_THREADS.store(count, Ordering::Relaxed);
    Ok(count)
}

/// Доступная RAM в байтах (приблизительно).
pub fn ram_bytes() -> Result<u64, LocalProviderError> {
    let sys = sysinfo::System::new_all();
    Ok(sys.available_memory())
}

/// RAM в гигабайтах.
pub fn ram_gb() -> Result<f64, LocalProviderError> {
    Ok(ram_bytes()? as f64 / (1024.0 * 1024.0 * 1024.0))
}
