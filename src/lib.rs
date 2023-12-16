/// This module contains the main functionality of the `xdiff` library.
pub mod cli;
pub mod config;

pub use config::{DiffConfig, DiffProfile, RequestProfile, ResponseProfile};

/// Represents additional arguments for the `xdiff` library.
#[derive(Debug)]
pub struct ExtraArgs {
    headers: Vec<(String, String)>,
    body: Vec<(String, String)>,
    query: Vec<(String, String)>,
}
