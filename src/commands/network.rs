use crate::commands::Handler;
use crate::config::Config;
pub struct NetworkHandler;
impl Handler for NetworkHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
