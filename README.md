# lite-bb (Lite Bitbucket CLI)

Because sometimes you want `gh`, but your company uses Bitbucket.

`lite-bb` is a lightweight, `gh`-style command-line tool for managing Bitbucket pull requests. It brings the simplicity and ergonomics of the GitHub CLI to Bitbucket — whether you're on **Bitbucket Cloud** or running **Bitbucket Server / Data Center** on-prem.

Built for developers who live in the terminal, CI/CD pipelines that need scriptable PR workflows, and LLM agents that benefit from structured `--json` output.

## Features

- **Familiar interface** — if you know `gh pr`, you already know `bb pr`. Same commands, same flags, same muscle memory.
- **Cloud + on-prem** — works with both Bitbucket Cloud and Bitbucket Server / Data Center. Auto-detects your instance type from the git remote.
- **Zero-config start** — auto-detects workspace, repo, and branch from your git context. Just `cd` into your repo and go.
- **Machine-readable output** — every view command supports `--json` for scripting, piping, and LLM agent consumption.
- **Credential verification** — `bb auth login` validates your token against the API before saving, so you catch auth issues immediately.
- **Easy install** — distributed as a pre-built binary via PyPI. No Rust toolchain needed — just `pip install lite-bb`.

## Install

The recommended way to install `lite-bb` is via pip or uv. This downloads a pre-built native binary for your platform — no Rust toolchain required.

```bash
pip install lite-bb
# or
uv pip install lite-bb
```

After installation, the `bb` command is available on your PATH.

<details>
<summary>Building from source</summary>

If you prefer to build from source, you'll need the Rust toolchain installed:

```bash
cargo install --path crates/cli
```

This compiles and installs the `bb` binary to `~/.cargo/bin/`.
</details>

## Quick Start

```bash
# 1. Authenticate (interactive — choose Cloud or Server, enter token)
bb auth login

# 2. List open PRs in the current repo
bb pr list

# 3. Create a PR from your current branch
bb pr create --title "feat: add user authentication"

# 4. View PR details (human-readable or JSON)
bb pr view 42
bb pr view 42 --json
```

## Authentication

`lite-bb` supports two authentication methods, matching how Bitbucket handles access:

- **Access Token** — a single token value (workspace, project, or repository token). This is the simplest option, similar to `gh auth login` with a personal access token.
- **App Password** — a username + app password pair. Useful when your organization requires app passwords for API access.

### Interactive Login

```bash
bb auth login
```

This walks you through an interactive setup:
1. Choose your Bitbucket instance — **Cloud** or **Server / Data Center**
2. For Server/DC, enter the server URL (auto-detected from your git remote if available)
3. Choose your credential type — **Access Token** or **App Password**
4. Enter your credentials
5. Credentials are verified against the API before saving

```bash
# Check your current auth status
bb auth status

# Remove saved credentials
bb auth logout
```

### Environment Variables

For CI/CD pipelines and scripting, you can set credentials via environment variables. These take priority over the config file.

| Variable | Description |
|----------|-------------|
| `BB_TOKEN` | Access token (used as Basic auth for Cloud, Bearer for Server/DC) |
| `BB_USERNAME` | Username for app password auth |
| `BB_APP_PASSWORD` | App password (used together with `BB_USERNAME`) |
| `BB_SERVER_URL` | Bitbucket Server/DC base URL (e.g. `https://bitbucket.company.com`) |
| `BB_CONFIG_DIR` | Override the config directory (default: `~/.config/bb/`) |

The config file is stored at `~/.config/bb/config.yml` and respects the `XDG_CONFIG_HOME` environment variable.

## Usage

All commands auto-detect the workspace and repository from your git remote. You can override this with `-R WORKSPACE/REPO` (or `-R PROJECT/REPO` for Server/DC).

### Listing Pull Requests

```bash
# List open PRs (default)
bb pr list

# Filter by state and limit results
bb pr list --state MERGED --limit 10
bb pr list --state DECLINED --limit 5
```

### Viewing a Pull Request

```bash
# Human-readable summary
bb pr view 42

# JSON output (for scripts, pipes, and LLM agents)
bb pr view 42 --json
```

### Creating a Pull Request

```bash
# Create from current branch (auto-detected) to default branch
bb pr create --title "feat: add login page"

# Specify description
bb pr create --title "feat: add login page" --body "Implements the login page with OAuth support"

# Specify source and destination branches explicitly
bb pr create --title "fix: typo in docs" --head my-branch --base main
```

### Editing a Pull Request

```bash
# Update title
bb pr edit 42 --title "new title"

# Update description
bb pr edit 42 --body "updated description"

# Change destination branch
bb pr edit 42 --base develop
```

### Merging

```bash
# Merge with default strategy
bb pr merge 42

# Merge with a specific strategy and commit message
bb pr merge 42 --strategy squash --message "squash: combine all commits"
```

### Code Review

```bash
# Approve a PR
bb pr review 42 --approve

# Request changes (unapprove)
bb pr review 42 --request-changes
```

### Comments

```bash
# Add a comment to a PR
bb pr comment 42 --body "Looks good! Just one minor suggestion on line 15."
```

### Diffs and CI Checks

```bash
# View the PR diff
bb pr diff 42

# View CI/CD pipeline status
bb pr checks 42

# CI checks as JSON
bb pr checks 42 --json
```

### Branch Operations

```bash
# Checkout a PR branch locally (fetches and switches)
bb pr checkout 42

# Close (decline) a PR
bb pr close 42

# Reopen a previously declined PR
bb pr reopen 42
```

## Command Reference

| Command | Description |
|---------|-------------|
| `bb auth login` | Authenticate with Bitbucket (Cloud or Server/DC) |
| `bb auth logout` | Remove saved credentials |
| `bb auth status` | Show current auth info and provider |
| `bb pr list` | List pull requests (filterable by state) |
| `bb pr view <id>` | View pull request details |
| `bb pr create` | Create a new pull request |
| `bb pr edit <id>` | Edit title, description, or destination branch |
| `bb pr merge <id>` | Merge a pull request |
| `bb pr close <id>` | Decline / close a pull request |
| `bb pr reopen <id>` | Reopen a declined pull request |
| `bb pr review <id>` | Approve or request changes |
| `bb pr comment <id>` | Add a comment to a pull request |
| `bb pr diff <id>` | View the pull request diff |
| `bb pr checks <id>` | View CI/CD status checks |
| `bb pr checkout <id>` | Fetch and checkout the PR branch locally |

## Bitbucket Server / Data Center

`lite-bb` fully supports Bitbucket Server and Data Center (on-prem) installations. The CLI automatically adapts its API calls, authentication method, and URL structure based on your configured provider.

**Key differences handled automatically:**

- **API endpoints** — Cloud uses `/2.0/repositories/{workspace}/{repo}`, Server/DC uses `/rest/api/1.0/projects/{project}/repos/{repo}`
- **Authentication** — Cloud tokens use Basic auth with `x-token-auth`, Server/DC personal access tokens use Bearer auth
- **Pagination** — Cloud uses `page/pagelen`, Server/DC uses `start/limit`
- **Diff format** — Cloud returns raw unified diff, Server/DC returns structured JSON (converted to unified diff automatically)
- **Git remotes** — parses both Cloud (`bitbucket.org`) and Server/DC remote URL formats (SSH with port, HTTPS with `/scm/` prefix, SCP-style)

No special flags are needed — just configure your server URL during `bb auth login` or via `BB_SERVER_URL`, and all commands work the same way.

## Development

```bash
cargo build            # build the CLI binary
cargo test             # run all unit tests
cargo run -- pr list   # run the CLI directly without installing
```

## License

MIT
