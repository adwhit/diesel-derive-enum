#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod common;
mod complex_join;
mod nullable;
#[cfg(feature = "postgres")]
mod pg_array;
#[cfg(feature = "postgres")]
mod pg_clone_impl;
#[cfg(feature = "postgres")]
mod pg_remote_type;
mod simple;
mod value_style;
