use crate::commands::Handler;
use crate::config::Config;
pub struct DatabaseHandler;
impl Handler for DatabaseHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
