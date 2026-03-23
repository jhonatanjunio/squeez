use crate::commands::Handler;
use crate::config::Config;
pub struct RuntimeHandler;
impl Handler for RuntimeHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
