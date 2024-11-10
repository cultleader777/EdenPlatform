#[macro_use]
extern crate lazy_static;

#[allow(dead_code, non_snake_case)]
pub mod database;
pub mod tests;
pub mod static_analysis;
pub mod codegen;
pub mod cli;
pub mod prom_metrics_dump;
