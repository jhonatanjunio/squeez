use crate::commands::Handler;
use crate::config::Config;
pub struct GenericHandler;
impl Handler for GenericHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
