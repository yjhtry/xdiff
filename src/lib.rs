/// This module contains the main functionality of the `xdiff` library.
pub mod cli;
pub mod config;
pub mod req;
pub mod utils;

pub use config::{DiffConfig, DiffProfile, ResponseProfile};
pub use req::RequestProfile;

/// Represents additional arguments for the `xdiff` library.
#[derive(Debug)]
pub struct ExtraArgs {
    headers: Vec<(String, String)>,
    body: Vec<(String, String)>,
    query: Vec<(String, String)>,
}
