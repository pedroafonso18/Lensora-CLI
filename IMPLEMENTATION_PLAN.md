# Lensora - Implementation Plan

## Goal

Build a Rust CLI named Lensora that reads uncommitted Git diffs, lets the user choose files or all files, and sends each selected change set to specialized LLM review agents defined in `agents/`.

## Product Decisions

- Language: Rust
- CLI framework: `clap`
- LLM providers: Anthropic and OpenAI
- Credentials and provider settings: `.env`
- Project config: TOML
- Ignore rules: defined in the config file
- Extra codebase context: defined in the config file
- Failure policy: stop execution on provider or agent failure
- Output: terminal summary, with optional file export

## Scope

### In scope

- Read uncommitted Git diff from the working tree
- Include staged changes as part of the input model
- Group changes by file
- Show an interactive file picker in the CLI
- Support choosing one file, multiple files, or all files
- Load agent prompts from `agents/`
- Orchestrate reviews through a main agent flow
- Consolidate results into a single final report
- Support config-driven ignore lists and repo preconditions

### Out of scope for the first version

- Git commit creation
- Automatic code changes or patch application
- Advanced caching
- Multi-repo support
- Plugin system for third-party agents

## Architecture

### CLI layer

- Parse command-line arguments
- Load config and environment variables
- Discover Git changes
- Present interactive selection
- Trigger review orchestration
- Render final output

### Domain layer

- Represent changed files and hunks
- Represent agent reviews and findings
- Represent merged final report
- Encapsulate ignore and filtering rules

### Infrastructure layer

- Git diff reader
- Config loader
- `.env` loader
- Anthropic client
- OpenAI client
- File system access for `agents/`

## Implementation Phases

### Phase 1 - Project skeleton

- Define the CLI entrypoint
- Add config loading
- Add environment variable loading
- Add a basic command structure
- Add initial error handling

### Phase 2 - Git diff discovery

- Read uncommitted changes from the working tree
- Read staged changes
- Group results by file
- Apply ignore rules from config

### Phase 3 - Interactive file selection

- Show altered files in a terminal prompt
- Let the user select one or more files
- Add an option to process all files
- Pass the selected files to the review pipeline

### Phase 4 - Agent orchestration

- Load prompt files from `agents/`
- Prepare the payload for each specialized review agent
- Send the selected diff plus repo context to the LLM provider
- Collect and normalize the agent outputs

### Phase 5 - Final report

- Build a terminal summary
- Show per-agent findings
- Show a consolidated summary
- Add optional file export for the report

### Phase 6 - Hardening

- Validate config on startup
- Improve error messages
- Add timeouts and response checks
- Add tests for config, diff parsing, and agent payload assembly

## Configuration Model

The config file should support:

- provider selection
- model selection per provider
- ignore paths or glob patterns
- repo preconditions or notes
- optional output path
- optional review defaults

## Suggested Next Deliverables

1. Create the config schema and loader.
2. Implement Git diff collection and file grouping.
3. Add interactive file selection.
4. Wire the agent prompt loader and provider client abstraction.
5. Produce the first terminal report.
