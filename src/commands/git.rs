use crate::commands::Handler;
use crate::config::Config;
pub struct GitHandler;
impl Handler for GitHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
