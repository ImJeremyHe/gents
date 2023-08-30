//! # gents
//! `gents` is a tool for generating `Typescript` files.
//! You can easily use `serde-json` to establish the communication between `Rust` and `Typescript`.
//! It is useful when you are developing
//! a web service or a wasm project.
//! ## Step1: Derive TS and set the `file_name`.
//! ```
//! use gents_derives::TS;
//!
//! #[derive(TS)]
//! #[ts(file_name = "person.ts")]
//! pub struct Person {
//!     pub age: u16,
//! }
//! ```
//!
//! ## Step2: Set your rename policy.
//! Currently, you can set `camelCase` using `rename_all`, or you
//! can rename each field by using `rename`.
//! ```
//! use gents_derives::TS;
//!
//! #[derive(TS)]
//! #[ts(file_name = "person.ts", rename_all = "camelCase")]
//! pub struct Person {
//!     pub age: u16,
//!     #[ts(rename="name")]
//!     pub en_name: String,
//! }
//! ```
//!
//! ## Step3: Register your root structs or enums
//! ```no_run
//! use gents::FileGroup;
//! use gents_derives::TS;
//! #[derive(TS)]
//! #[ts(file_name = "person.ts")]
//! pub struct Person{}
//!
//! fn main() {
//!     let mut g = FileGroup::new();
//!     g.add::<Person>();
//!     g.gen_files("outdir", false); // false for not generating index.ts
//! }
//! ```
//! `.add` adds the target and its dependencies into the `FileGroup` and their files
//! will be generated in the same time.
//!
//! ## Step4: Run the binary
//!

mod descriptor;
mod file_generator;
mod utils;

pub use descriptor::*;
pub use file_generator::*;
