#![cfg_attr(feature = "unstable-toolchain", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable-toolchain", allow(internal_features))]

//! # Uring IO

pub mod arena;
pub mod completion;
pub mod operator;
pub mod platform;
pub mod register;
pub mod shared;
pub mod submission;
pub mod uring;
