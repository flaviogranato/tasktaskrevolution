#![allow(non_snake_case)]
#![allow(clippy::module_inception)]
#![allow(dead_code)]

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;

pub use domain::task::Task; 