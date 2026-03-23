use crate::config::Config;

pub trait Handler {
    fn compress(&self, cmd: &str, lines: Vec<String>, config: &Config) -> Vec<String>;
}

pub mod wrap;
pub mod filter_stdin;
pub mod git;
pub mod docker;
pub mod package_mgr;
pub mod test_runner;
pub mod typescript;
pub mod build;
pub mod cloud;
pub mod database;
pub mod network;
pub mod fs;
pub mod runtime;
pub mod generic;
