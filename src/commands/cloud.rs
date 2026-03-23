use crate::commands::Handler;
use crate::config::Config;
pub struct CloudHandler;
impl Handler for CloudHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
