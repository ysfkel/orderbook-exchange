// /// A lock-free data structure enables multiple threads to access it
// /// concurrently without using locks.
// /// Insstead, it relies on atomic operations, low-level CPU instructions that execute indivisibly,
// /// ensuring thread safety without blocking

// use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicUsize};
// use crossbeam_utils::CachePadded;

//  struct RingBuffer<T, const N: usize> {
//     values: [Slot<T>; N],
//     head: CachePadded<AtomicUsize>
//  }

//  pub struct Slot<T> {
//     value: UnsafeCell<MaybeUninit<T>>, //next read index
//     sequence: CachePadded<AtomicUsize> //next write index
//  }

//  unsafe impl<T, const N: usize> Send for RingBuffer<T, N>{}
//  unsafe impl<T, const N: usize> Sync for RingBuffer<T, N>{}

//  impl<T, const N: usize> RingBuffer<T> for RingBuffer<T, N> {

//  }
