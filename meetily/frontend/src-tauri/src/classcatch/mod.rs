//! Classroom additions to Meetily Community Edition. Recording remains owned by Meetily.
pub mod core;
mod prompts;
mod transport;
mod dsp;
pub mod cloud;
pub mod service;
pub mod history;
pub mod repair;
pub mod local;
pub use service::*;

pub fn cloud_enabled() -> bool { true }
