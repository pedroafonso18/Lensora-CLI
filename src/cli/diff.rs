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

    let staged_paths = git_paths(
        &repo_root,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            "-z",
            "--cached",
            "--diff-filter=ACDM",
        ],
    )?;
    let unstaged_paths = git_paths(
        &repo_root,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            "-z",
            "--diff-filter=ACDM",
        ],
    )?;
    let untracked_paths: BTreeSet<_> = git_paths(
        &repo_root,
        &["ls-files", "-z", "--others", "--exclude-standard"],
    )?
    .into_iter()
    .collect();

    let mut files = BTreeMap::<PathBuf, ChangedFile>::new();

    for path in staged_paths
        .into_iter()
        .chain(unstaged_paths)
        .chain(untracked_paths.iter().cloned())
    {
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
        let staged = tracked_diff(&repo_root, &file.path, true)?;
        let unstaged = if file.untracked {
            untracked_diff(&repo_root, &file.path)?
        } else {
            tracked_diff(&repo_root, &file.path, false)?
        };

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
    let mut command = Command::new("git");
    command.args(["rev-parse", "--show-toplevel"]);
    let mut root = git_output(command, false)?;
    if root.last() == Some(&b'\n') {
        root.pop();
    }
    path_from_git(&root)
}

#[cfg(unix)]
fn path_from_git(bytes: &[u8]) -> anyhow::Result<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    Ok(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
fn path_from_git(bytes: &[u8]) -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(std::str::from_utf8(bytes)?))
}

fn git_paths(repo_root: &Path, args: &[&str]) -> anyhow::Result<Vec<PathBuf>> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_root).args(args);
    git_output(command, false)?
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(path_from_git)
        .collect()
}

fn tracked_diff(repo_root: &Path, path: &Path, staged: bool) -> anyhow::Result<String> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_root).args([
        "--literal-pathspecs",
        "diff",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "--unified=3",
    ]);
    if staged {
        command.arg("--cached");
    }
    command.arg("--").arg(path);
    Ok(String::from_utf8_lossy(&git_output(command, false)?).into_owned())
}

fn untracked_diff(repo_root: &Path, path: &Path) -> anyhow::Result<String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo_root)
        .args([
            "diff",
            "--no-index",
            "--no-ext-diff",
            "--no-textconv",
            "--",
            "/dev/null",
        ])
        .arg(Path::new(".").join(path));
    Ok(String::from_utf8_lossy(&git_output(command, true)?).into_owned())
}

fn git_output(mut command: Command, allow_diff_exit: bool) -> anyhow::Result<Vec<u8>> {
    let output = command.output()?;
    if output.status.success()
        || (allow_diff_exit
            && output.status.code() == Some(1)
            && (output.stderr.is_empty() || !output.stdout.is_empty()))
    {
        return Ok(output.stdout);
    }
    anyhow::bail!(
        "Git failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    );
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
