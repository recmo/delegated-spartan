use core::sync::atomic::{AtomicUsize, Ordering};
use humantime::format_duration;
use libspartan::{Instance, NIZKGens, NIZK};
use merlin::Transcript;
use rayon::ThreadPoolBuilder;
use size::Size;
use std::{
    alloc::{GlobalAlloc, System as SystemAllocator},
    time::Instant,
};

pub struct MeasuringAllocator<Inner: GlobalAlloc> {
    inner: Inner,
    total: AtomicUsize,
    max:   AtomicUsize,
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
static ALLOCATOR: MeasuringAllocator<SystemAllocator> = MeasuringAllocator {
    inner: SystemAllocator,
    total: AtomicUsize::new(0),
    max:   AtomicUsize::new(0),
};

fn main() {
    // Force the library to use a single thread
    ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .unwrap();

    // specify the size of an R1CS instance
    let num_vars = 2 << 20;
    let num_cons = 2 << 20;
    let num_inputs = 10;

    // produce public parameters
    println!("generating public parameters...");
    let gens = NIZKGens::new(num_cons, num_vars, num_inputs);

    // ask the library to produce a synthentic R1CS instance
    println!("generating synthetic R1CS instance...");
    let (inst, vars, inputs) = Instance::produce_synthetic_r1cs(num_cons, num_vars, num_inputs);

    // produce a proof of satisfiability
    ALLOCATOR.max.store(0, Ordering::SeqCst);
    println!("generating proof of satisfiability...");
    let start = Instant::now();
    let mut prover_transcript = Transcript::new(b"nizk_example");
    let proof = NIZK::prove(&inst, vars, &inputs, &gens, &mut prover_transcript);
    let duration = start.elapsed();
    let proof_size = Size::from_bytes(bincode::serialized_size(&proof).unwrap());
    let max_mem = Size::from_bytes(ALLOCATOR.max.load(Ordering::SeqCst));
    println!(
        "proof size: {} time: {} max mem: {}",
        proof_size,
        format_duration(duration),
        max_mem
    );

    // verify the proof of satisfiability
    println!("verifying proof of satisfiability...");
    let start = Instant::now();
    let mut verifier_transcript = Transcript::new(b"nizk_example");
    assert!(proof
        .verify(&inst, &inputs, &mut verifier_transcript, &gens)
        .is_ok());
    let duration = start.elapsed();
    println!(
        "proof verification successful! in {}",
        format_duration(duration)
    );
}
