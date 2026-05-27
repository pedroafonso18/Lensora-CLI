use std::path::PathBuf;

use serde_json::Value;

use super::agent::load_agents;
use super::config::LensoraConfig;
use super::diff::ChangedFile;
use super::provider::{ProviderClient, ReviewProvider, ReviewRequest};

#[derive(Debug)]
pub struct ReviewReport {
    pub agent_results: Vec<AgentResult>,
}

#[derive(Debug)]
pub struct AgentResult {
    pub agent_name: String,
    pub status: String,
    pub summary: String,
}

pub fn run_review(config: &LensoraConfig, selected_files: &[ChangedFile]) -> anyhow::Result<ReviewReport> {
    let agent_dir = PathBuf::from("agents");
    let agents = load_agents(&agent_dir)?;
    let provider = ProviderClient::from_config(&config.provider)?;

    let explanation = build_explanation(config, selected_files);
    let code = build_diff_payload(selected_files);

    let mut agent_results = Vec::new();

    for agent in agents {
        let request = ReviewRequest {
            model: config.provider.model.clone(),
            system_prompt: agent.prompt.clone(),
            user_prompt: build_user_prompt(&code, &explanation),
        };

        let raw_output = provider.review(&agent, &request)?;
        let parsed_output = parse_review_json(&raw_output);
        let summary = summarize_output(&parsed_output, &raw_output);

        agent_results.push(AgentResult {
            agent_name: agent.name.clone(),
            status: "ok".to_string(),
            summary,
        });
    }

    Ok(ReviewReport { agent_results })
}

fn build_explanation(config: &LensoraConfig, selected_files: &[ChangedFile]) -> String {
    let mut sections = Vec::new();

    if !config.repo.preconditions.is_empty() {
        sections.push(format!("Repo preconditions:\n- {}", config.repo.preconditions.join("\n- ")));
    }

    sections.push(format!(
        "Selected files: {}",
        selected_files
            .iter()
            .map(|file| file.path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    sections.push("Review the provided diff and return a strict JSON object with summary and findings.".to_string());

    sections.join("\n\n")
}

fn build_diff_payload(selected_files: &[ChangedFile]) -> String {
    let mut output = String::new();

    for file in selected_files {
        output.push_str(&format!("FILE: {}\n", file.path.display()));

        if let Some(staged) = &file.staged_diff {
            output.push_str("STAGED DIFF:\n");
            output.push_str(staged);
            if !staged.ends_with('\n') {
                output.push('\n');
            }
        }

        if let Some(unstaged) = &file.unstaged_diff {
            output.push_str("UNSTAGED DIFF:\n");
            output.push_str(unstaged);
            if !unstaged.ends_with('\n') {
                output.push('\n');
            }
        }

        output.push('\n');
    }

    output
}

fn build_user_prompt(code: &str, explanation: &str) -> String {
    format!(
        "code:\n```diff\n{}\n```\n\nexplanation:\n{}",
        code, explanation
    )
}

fn summarize_output(parsed_output: &Option<Value>, raw_output: &str) -> String {
    if let Some(parsed) = parsed_output {
        if let Some(summary) = parsed.get("summary").and_then(|summary| summary.as_str()) {
            return summary.trim().to_string();
        }

        if let Some(findings) = parsed.get("findings").and_then(|findings| findings.as_array()) {
            return format!("{} finding(s)", findings.len());
        }
    }

    raw_output
        .lines()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with("```") && line.trim() != "json")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn parse_review_json(raw_output: &str) -> Option<Value> {
    let trimmed = raw_output.trim();

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Some(value);
    }

    if let Some(stripped) = strip_json_fences(trimmed) {
        if let Ok(value) = serde_json::from_str::<Value>(stripped) {
            return Some(value);
        }
    }

    None
}

fn strip_json_fences(raw_output: &str) -> Option<&str> {
    let without_opening = raw_output.strip_prefix("```json")?.trim_start();
    let without_closing = without_opening.strip_suffix("```")?.trim_end();
    Some(without_closing)
}

#[cfg(test)]
mod tests {
    use super::build_user_prompt;

    #[test]
    fn builds_prompt_with_code_and_explanation() {
        let prompt = build_user_prompt("diff body", "repo notes");

        assert!(prompt.contains("code:"));
        assert!(prompt.contains("diff body"));
        assert!(prompt.contains("explanation:"));
        assert!(prompt.contains("repo notes"));
    }
}