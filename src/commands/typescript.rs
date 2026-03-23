use crate::commands::Handler;
use crate::config::Config;
pub struct TypescriptHandler;
impl Handler for TypescriptHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
