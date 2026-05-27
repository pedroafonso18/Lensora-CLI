# Lensora CLI

Lensora is a Rust CLI for reviewing uncommitted Git diffs with specialized LLM agents.

## Current status

The project currently includes the CLI skeleton and TOML config loader.

## Run

```bash
cargo run -- --print-config
```

## Configuration

Copy `lensora.example.toml` to `lensora.toml` and adjust the provider, API key, ignore rules, and repo notes. The default Anthropic model is `claude-sonnet-4-6`.

You can set the provider key directly in `lensora.toml` with `api_key`. If you omit it, Lensora will fall back to `ANTHROPIC_API_KEY` for Anthropic or `OPENAI_API_KEY` for OpenAI.