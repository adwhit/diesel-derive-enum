#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate diesel;

mod common;
mod nullable;
mod pg_array;
mod rename;
mod simple;
mod value_style;
