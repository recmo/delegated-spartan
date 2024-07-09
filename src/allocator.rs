use core::sync::atomic::{AtomicUsize, Ordering};
use std::alloc::{GlobalAlloc, System as SystemAllocator};

pub struct MeasuringAllocator<Inner: GlobalAlloc> {
    inner: Inner,
    total: AtomicUsize,
    max:   AtomicUsize,
}

impl<Inner: GlobalAlloc> MeasuringAllocator<Inner> {
    pub fn reset(&self) {
        self.max.store(self.total.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn max(&self) -> usize {
        self.max.load(Ordering::Relaxed)
    }
}

unsafe impl<Inner: GlobalAlloc> GlobalAlloc for MeasuringAllocator<Inner> {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        self.total.fetch_add(layout.size(), Ordering::Relaxed);
        self.max
            .fetch_max(self.total.load(Ordering::Relaxed), Ordering::Relaxed);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        self.inner.dealloc(ptr, layout);
        self.total.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
pub static ALLOCATOR: MeasuringAllocator<SystemAllocator> = MeasuringAllocator {
    inner: SystemAllocator,
    total: AtomicUsize::new(0),
    max:   AtomicUsize::new(0),
};
