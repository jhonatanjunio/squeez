use crate::commands::Handler;
use crate::config::Config;
pub struct FsHandler;
impl Handler for FsHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
