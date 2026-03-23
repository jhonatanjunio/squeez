use crate::commands::Handler;
use crate::config::Config;
pub struct DockerHandler;
impl Handler for DockerHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
