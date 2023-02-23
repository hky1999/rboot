use core::fmt;
use core::ops::{Deref, DerefMut};

pub use uefi::table::boot::{MemoryAttribute, MemoryDescriptor, MemoryType};

// const PAGE_SIZE: u64 = 4096;

const MAX_MEMORY_MAP_SIZE: usize = 64;

/// A map of the physical memory regions of the underlying machine.
#[repr(C)]
pub struct MemoryMap {
    entries: [MemoryDescriptor; MAX_MEMORY_MAP_SIZE],
    // u64 instead of usize so that the structure layout is platform
    // independent
    next_entry_index: u64,
}

impl MemoryMap {
    pub fn new() -> Self {
        MemoryMap {
            entries: [MemoryDescriptor::default(); MAX_MEMORY_MAP_SIZE],
            next_entry_index: 0,
        }
    }

    pub fn add_region(&mut self, region: MemoryDescriptor) {
        assert!(
            self.next_entry_index() < MAX_MEMORY_MAP_SIZE,
            "too many memory regions in memory map"
        );
        self.entries[self.next_entry_index()] = region;
        self.next_entry_index += 1;
        self.sort();
    }

    pub fn sort(&mut self) {
        use core::cmp::Ordering;

        self.entries.sort_unstable_by(|r1, r2| {
            if r1.page_count == 0 {
                Ordering::Greater
            } else if r2.page_count == 0 {
                Ordering::Less
            } else {
                let ordering = r1.phys_start.cmp(&r2.phys_start);
                if ordering == Ordering::Equal {
                    r1.page_count.cmp(&r2.page_count)
                } else {
                    ordering
                }
            }
        });
        if let Some(first_zero_index) = self.entries.iter().position(|r| r.page_count == 0) {
            self.next_entry_index = first_zero_index as u64;
        }
    }

    fn next_entry_index(&self) -> usize {
        self.next_entry_index as usize
    }
}

impl Deref for MemoryMap {
    type Target = [MemoryDescriptor];

    fn deref(&self) -> &Self::Target {
        &self.entries[0..self.next_entry_index()]
    }
}

impl DerefMut for MemoryMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let next_index = self.next_entry_index();
        &mut self.entries[0..next_index]
    }
}

impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
