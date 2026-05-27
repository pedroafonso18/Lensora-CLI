use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LensoraConfig {
    pub provider: ProviderConfig,
    pub ignore: IgnoreConfig,
    pub repo: RepoConfig,
    pub output: OutputConfig,
}

impl LensoraConfig {
    pub fn load_or_default(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file at {}", path.display()))?;

        let config = toml::from_str(&contents)
            .with_context(|| format!("failed to parse TOML config at {}", path.display()))?;

        Ok(config)
    }

    pub fn redacted_for_display(&self) -> Self {
        let mut config = self.clone();

        if config.provider.api_key.is_some() {
            config.provider.api_key = Some("<redacted>".to_string());
        }

        config
    }
}

impl Default for LensoraConfig {
    fn default() -> Self {
        Self {
            provider: ProviderConfig::default(),
            ignore: IgnoreConfig::default(),
            repo: RepoConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    pub name: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            api_key: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct IgnoreConfig {
    pub paths: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self { paths: Vec::new() }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RepoConfig {
    pub preconditions: Vec<String>,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            preconditions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    pub path: Option<String>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { path: None }
    }
}

trait Context<T> {
    fn with_context<F>(self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|error| anyhow::Error::new(error).context(f()))
    }
}