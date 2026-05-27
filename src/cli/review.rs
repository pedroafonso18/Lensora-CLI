use std::path::PathBuf;
use std::fs;

use serde_json::Value;

use super::agent::load_agents;
use super::config::LensoraConfig;
use super::diff::ChangedFile;
use super::provider::{ProviderClient, ReviewProvider, ReviewRequest};

#[derive(Debug)]
pub struct ReviewReport {
    pub reviewed_files: Vec<String>,
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

    Ok(ReviewReport {
        reviewed_files: selected_files
            .iter()
            .map(|file| file.path.display().to_string())
            .collect(),
        agent_results,
    })
}

pub fn export_report(output_path: &str, report: &ReviewReport) -> anyhow::Result<()> {
    fs::write(output_path, render_report(report))?;
    Ok(())
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
        "code:\n```diff\n{}\n```\n\nexplanation:\n{}\n\nReturn only the review JSON. Do not propose patches or code changes.",
        code, explanation
    )
}

fn render_report(report: &ReviewReport) -> String {
    let mut output = String::new();

    output.push_str("# Lensora Review Report\n\n");

    output.push_str("## Files Reviewed\n\n");
    for file in &report.reviewed_files {
        output.push_str(&format!("- {}\n", file));
    }
    output.push_str("\n");

    for agent_result in &report.agent_results {
        output.push_str(&format!("## {}\n\n", agent_result.agent_name));
        output.push_str(&format!("Status: {}\n\n", agent_result.status));
        output.push_str(&format!("Summary: {}\n\n", agent_result.summary));
    }

    output
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
    use super::{build_user_prompt, render_report, AgentResult, ReviewReport};

    #[test]
    fn builds_prompt_with_code_and_explanation() {
        let prompt = build_user_prompt("diff body", "repo notes");

        assert!(prompt.contains("code:"));
        assert!(prompt.contains("diff body"));
        assert!(prompt.contains("explanation:"));
        assert!(prompt.contains("repo notes"));
        assert!(prompt.contains("Do not propose patches or code changes."));
    }

    #[test]
    fn renders_report_markdown() {
        let report = ReviewReport {
            reviewed_files: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            agent_results: vec![AgentResult {
                agent_name: "bug".to_string(),
                status: "ok".to_string(),
                summary: "1 finding(s)".to_string(),
            }],
        };

        let rendered = render_report(&report);

        assert!(rendered.contains("# Lensora Review Report"));
        assert!(rendered.contains("## Files Reviewed"));
        assert!(rendered.contains("- src/main.rs"));
        assert!(rendered.contains("- src/lib.rs"));
        assert!(rendered.contains("## bug"));
        assert!(rendered.contains("Summary: 1 finding(s)"));
    }
}