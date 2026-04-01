use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Lock-free per-row dirty tracking.
///
/// The PTY read thread sets bits atomically when rows change.
/// The render thread reads and clears them atomically.
/// Supports up to 256 rows (4 x 64-bit atomic words).
pub struct AtomicDamage {
    rows: [AtomicU64; 4],
    full: AtomicBool,
}

impl AtomicDamage {
    pub fn new() -> Self {
        Self {
            rows: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            full: AtomicBool::new(true), // Initial full damage
        }
    }

    /// Mark a single row as dirty.
    pub fn mark_row(&self, row: u16) {
        let word = (row / 64) as usize;
        let bit = row % 64;
        if word < self.rows.len() {
            self.rows[word].fetch_or(1 << bit, Ordering::Release);
        }
    }

    /// Mark all rows as dirty (e.g., after resize or scroll).
    pub fn mark_full(&self) {
        self.full.store(true, Ordering::Release);
    }

    /// Check if any damage exists without clearing.
    pub fn has_damage(&self) -> bool {
        if self.full.load(Ordering::Acquire) {
            return true;
        }
        self.rows
            .iter()
            .any(|w| w.load(Ordering::Acquire) != 0)
    }

    /// Take all damage, returning which rows are dirty. Clears flags.
    pub fn take(&self) -> DamageSnapshot {
        let full = self.full.swap(false, Ordering::AcqRel);
        let rows = [
            self.rows[0].swap(0, Ordering::AcqRel),
            self.rows[1].swap(0, Ordering::AcqRel),
            self.rows[2].swap(0, Ordering::AcqRel),
            self.rows[3].swap(0, Ordering::AcqRel),
        ];
        DamageSnapshot { full, rows }
    }
}

impl Default for AtomicDamage {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of damage state at a point in time.
pub struct DamageSnapshot {
    pub full: bool,
    rows: [u64; 4],
}

impl DamageSnapshot {
    /// Check if a specific row is dirty.
    pub fn is_row_dirty(&self, row: u16) -> bool {
        if self.full {
            return true;
        }
        let word = (row / 64) as usize;
        let bit = row % 64;
        if word < self.rows.len() {
            self.rows[word] & (1 << bit) != 0
        } else {
            false
        }
    }
}
