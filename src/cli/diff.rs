use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub staged_diff: Option<String>,
    pub unstaged_diff: Option<String>,
    pub untracked: bool,
}

#[derive(Debug, Clone)]
pub struct RepoDiff {
    pub files: Vec<ChangedFile>,
}

pub fn collect_repo_diff(ignore_patterns: &[String]) -> anyhow::Result<RepoDiff> {
    let repo_root = repo_root()?;

    let staged_paths = git_lines(&repo_root, &["diff", "--name-only", "--cached", "--diff-filter=ACMR"])?;
    let unstaged_paths = git_lines(&repo_root, &["diff", "--name-only", "--diff-filter=ACMR"])?;
    let untracked_paths = git_lines(&repo_root, &["ls-files", "--others", "--exclude-standard"])?;
    let untracked_paths: BTreeSet<PathBuf> = untracked_paths.into_iter().map(PathBuf::from).collect();

    let mut files = BTreeMap::<PathBuf, ChangedFile>::new();

    for path in staged_paths
        .into_iter()
        .chain(unstaged_paths.into_iter())
        .chain(untracked_paths.iter().map(|path| path.to_string_lossy().to_string()))
    {
        let path = PathBuf::from(path);

        if should_ignore(&path, ignore_patterns) {
            continue;
        }

        files.entry(path.clone()).or_insert_with(|| ChangedFile {
            untracked: untracked_paths.contains(&path),
            path,
            staged_diff: None,
            unstaged_diff: None,
        });
    }

    for file in files.values_mut() {
        if file.untracked {
            file.unstaged_diff = Some(untracked_diff(&repo_root, &file.path)?);
            continue;
        }

        let path_arg = file.path.to_string_lossy().to_string();
        let staged = git_text(&repo_root, &["diff", "--cached", "--unified=3", "--", &path_arg])?;
        let unstaged = git_text(&repo_root, &["diff", "--unified=3", "--", &path_arg])?;

        if !staged.trim().is_empty() {
            file.staged_diff = Some(staged);
        }

        if !unstaged.trim().is_empty() {
            file.unstaged_diff = Some(unstaged);
        }
    }

    Ok(RepoDiff {
        files: files.into_values().collect(),
    })
}

fn repo_root() -> anyhow::Result<PathBuf> {
    let output = Command::new("git").args(["rev-parse", "--show-toplevel"]).output()?;

    if !output.status.success() {
        return Ok(std::env::current_dir()?);
    }

    let root = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(root))
}

fn git_lines(repo_root: &Path, args: &[&str]) -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn git_text(repo_root: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()?;

    if !output.status.success() {
        return Ok(String::new());
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn untracked_diff(repo_root: &Path, path: &Path) -> anyhow::Result<String> {
    let path_arg = path.to_string_lossy().to_string();
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["diff", "--no-index", "--", "/dev/null", &path_arg])
        .output()?;

    if output.status.success() || output.status.code() == Some(1) {
        return Ok(String::from_utf8(output.stdout)?);
    }

    Ok(String::new())
}

fn should_ignore(path: &Path, ignore_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    ignore_patterns.iter().any(|pattern| {
        let trimmed = pattern.trim();

        if trimmed.is_empty() {
            return false;
        }

        let normalized = trimmed.trim_start_matches("./");
        path_str == normalized || path_str.starts_with(normalized) || path_str.contains(normalized)
    })
}

#[cfg(test)]
mod tests {
    use super::should_ignore;
    use std::path::PathBuf;

    #[test]
    fn ignores_matching_prefixes() {
        let path = PathBuf::from("target/debug/app");
        let patterns = vec!["target/".to_string()];

        assert!(should_ignore(&path, &patterns));
    }

    #[test]
    fn keeps_non_matching_paths() {
        let path = PathBuf::from("src/main.rs");
        let patterns = vec!["target/".to_string()];

        assert!(!should_ignore(&path, &patterns));
    }
}