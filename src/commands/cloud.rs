use crate::commands::Handler;
use crate::config::Config;
use crate::strategies::{smart_filter, toon, truncation};

pub struct CloudHandler;

impl Handler for CloudHandler {
    fn compress(&self, cmd: &str, lines: Vec<String>, _config: &Config) -> Vec<String> {
        // TOON re-encoding for JSON-array outputs (gh --json, kubectl -o json,
        // aws --output json, gcloud --format=json, az --output json). Lossless
        // when uniform; falls back to the standard pipeline otherwise.
        if looks_like_json_output(cmd) {
            let joined = lines.join("\n");
            if let Some(toon_out) = toon::try_to_toon(&joined) {
                return toon_out.lines().map(String::from).collect();
            }
        }
        let lines = smart_filter::apply(lines);
        let filtered: Vec<String> = lines
            .into_iter()
            .filter(|l| !l.starts_with("---") && !l.trim().is_empty())
            .collect();
        truncation::apply(filtered, 100, truncation::Keep::Head)
    }
}

/// Heuristic: does this invocation request JSON output?
fn looks_like_json_output(cmd: &str) -> bool {
    let lc = cmd.to_ascii_lowercase();
    lc.contains("--json")
        || lc.contains("-o json")
        || lc.contains("-ojson")
        || lc.contains("--output json")
        || lc.contains("--output=json")
        || lc.contains("--format json")
        || lc.contains("--format=json")
        || lc.contains("-f json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_json_request_flags() {
        assert!(looks_like_json_output("gh pr list --json number,title"));
        assert!(looks_like_json_output("kubectl get pods -o json"));
        assert!(looks_like_json_output("kubectl get pods -ojson"));
        assert!(looks_like_json_output("aws ec2 describe-instances --output json"));
        assert!(looks_like_json_output("aws ec2 describe-instances --output=json"));
        assert!(looks_like_json_output("gcloud compute instances list --format=json"));
        assert!(!looks_like_json_output("gh pr list"));
        assert!(!looks_like_json_output("kubectl get pods -o yaml"));
    }
}
