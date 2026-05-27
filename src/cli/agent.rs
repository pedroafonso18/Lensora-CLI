use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AgentSpec {
    pub name: String,
    pub prompt: String,
}

pub fn load_agents(agent_dir: impl AsRef<Path>) -> anyhow::Result<Vec<AgentSpec>> {
    let agent_dir = agent_dir.as_ref();
    let mut agents = Vec::new();

    if !agent_dir.exists() {
        anyhow::bail!("agent directory not found at {}", agent_dir.display());
    }

    for entry in fs::read_dir(agent_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let prompt = fs::read_to_string(&path)?;
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("agent")
            .to_string();

        agents.push(AgentSpec {
            name,
            prompt,
        });
    }

    agents.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(agents)
}

#[cfg(test)]
mod tests {
    use super::load_agents;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_md_agents_from_directory() {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("lensora-agent-test-{}-{}", std::process::id(), unique_suffix));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("bug.md"), "bug prompt").unwrap();
        fs::write(base.join("skip.txt"), "ignore").unwrap();

        let agents = load_agents(&base).unwrap();

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "bug");
        assert_eq!(agents[0].prompt, "bug prompt");

        let _ = fs::remove_dir_all(&base);
    }
}