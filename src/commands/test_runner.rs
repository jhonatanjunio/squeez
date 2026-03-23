use crate::commands::Handler;
use crate::config::Config;
pub struct TestRunnerHandler;
impl Handler for TestRunnerHandler {
    fn compress(&self, _cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> { lines }
}
