use std;

#[cfg(target_pointer_width = "64")]
pub type AtomicI64 = std::sync::atomic::AtomicIsize;
#[cfg(not(target_pointer_width = "64"))]
pub type AtomicI64 = std::sync::atomic::AtomicI64;

#[cfg(target_pointer_width = "64")]
pub type AtomicU64 = std::sync::atomic::AtomicUsize;
#[cfg(not(target_pointer_width = "64"))]
pub type AtomicU64 = std::sync::atomic::AtomicU64;

#[cfg(target_pointer_width = "64")]
pub type FakeU64 = usize;
#[cfg(not(target_pointer_width = "64"))]
pub type FakeU64 = u64;

#[cfg(target_pointer_width = "64")]
pub type FakeI64 = isize;
#[cfg(not(target_pointer_width = "64"))]
pub type FakeI64 = i64;

pub type AtomicPtr<T> = std::sync::atomic::AtomicPtr<T>;
pub type AtomicBool = std::sync::atomic::AtomicBool;
pub type AtomicIsize = std::sync::atomic::AtomicIsize;
pub type AtomicUsize = std::sync::atomic::AtomicUsize;
pub use std::sync::atomic::Ordering;
