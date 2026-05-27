use clap::Parser;
use colored::Colorize;

pub mod agent;
pub mod diff;
pub mod config;
pub mod provider;
pub mod selection;
pub mod review;

#[derive(Debug, Parser)]
#[command(
    name = "lensora",
    version,
    about = "Review uncommitted Git diffs with specialized LLM agents"
)]
struct Cli {
    #[arg(short, long, value_name = "FILE", default_value = "lensora.toml")]
    config: std::path::PathBuf,

    #[arg(long)]
    print_config: bool,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = config::LensoraConfig::load_or_default(&cli.config)?;

    if cli.print_config {
        println!("{:#?}", config.redacted_for_display());
        return Ok(());
    }

    println!("{}", "Lensora initialized.".bold().cyan());
    println!("{} {}", "Config path:".dimmed(), cli.config.display().to_string().green());

    let repo_diff = diff::collect_repo_diff(&config.ignore.paths)?;

    if repo_diff.files.is_empty() {
        println!("No uncommitted changes found.");
        return Ok(());
    }

    println!("{} {}", "Found changed files:".bold().cyan(), repo_diff.files.len().to_string().yellow());

    let selected_files = selection::choose_files(&repo_diff.files)?;

    println!();
    println!("{} {}", "Selected for review:".bold().cyan(), selected_files.len().to_string().yellow());
    for file in &selected_files {
        println!("  {}", file.path.display().to_string().green());
    }

    let report = review::run_review(&config, &selected_files)?;

    println!();
    println!("{}", "Review results".bold().cyan());
    for (index, agent_result) in report.agent_results.iter().enumerate() {
        if index > 0 {
            println!();
        }

        println!("{}", agent_result.agent_name.bold().blue());
        println!("  {} {}", "status:".dimmed(), agent_result.status.green());
        println!("  {}", agent_result.summary);
    }

    Ok(())
}