#![no_std]
extern crate alloc;

pub mod class;
pub mod error;
pub mod helper;
pub mod issuer;
pub mod nft;
pub mod hash;
pub mod mol;


cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use ckb_types::{self, molecule};
        pub use std::vec;
        pub use std::borrow::ToOwned;
    } else  if #[cfg(feature = "no-std")] {
        pub use ckb_std::ckb_types;
        pub use molecule;
        extern crate alloc;
        pub use alloc::vec;
        pub use alloc::borrow::ToOwned;
    }
}