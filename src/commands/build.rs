use crate::commands::Handler;
use crate::config::Config;
pub struct BuildHandler;
impl Handler for BuildHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
