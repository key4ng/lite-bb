# lite-bb

Because sometimes you want `gh`, but your company uses Bitbucket. `lite-bb` is a gh-style minimal CLI for pull request creation, diffs, reviews, and comments — designed for humans, scripts, and LLM agents.

## Install

```bash
# Via pip/uv (recommended — pre-built binary, no Rust toolchain needed)
pip install lite-bb
# or
uv pip install lite-bb

# From source
cargo install --path crates/cli
```

## Auth

```bash
# Access token (simplest — single value, like gh)
bb auth login

# Check status
bb auth status

# Log out
bb auth logout
```

Env var overrides: `BB_TOKEN`, or `BB_USERNAME` + `BB_APP_PASSWORD`.

## Usage

### Pull Requests

```bash
# List open PRs
bb pr list
bb pr list --state MERGED --limit 10

# View a PR
bb pr view 42
bb pr view 42 --json

# Create a PR (auto-detects current branch)
bb pr create --title "feat: add login" --body "Description here"
bb pr create --title "fix: typo" --head my-branch --base main

# Edit a PR
bb pr edit 42 --title "new title" --body "updated description"

# Merge
bb pr merge 42
bb pr merge 42 --strategy squash --message "squash merge"

# Review
bb pr review 42 --approve
bb pr review 42 --request-changes

# Comment
bb pr comment 42 --body "Looks good!"

# Diff
bb pr diff 42

# CI/CD checks
bb pr checks 42
bb pr checks 42 --json

# Checkout PR branch locally
bb pr checkout 42

# Close / Reopen
bb pr close 42
bb pr reopen 42
```

All PR commands support `-R WORKSPACE/REPO` to override auto-detection from git remote.

## Commands

| Command | Description |
|---------|-------------|
| `bb auth login` | Authenticate with Bitbucket |
| `bb auth logout` | Log out |
| `bb auth status` | View auth status |
| `bb pr list` | List pull requests |
| `bb pr view <id>` | View a pull request |
| `bb pr create` | Create a pull request |
| `bb pr edit <id>` | Edit title, description, or base |
| `bb pr merge <id>` | Merge a pull request |
| `bb pr close <id>` | Decline/close a pull request |
| `bb pr reopen <id>` | Reopen a declined pull request |
| `bb pr review <id>` | Approve or request changes |
| `bb pr comment <id>` | Add a comment |
| `bb pr diff <id>` | View PR diff |
| `bb pr checks <id>` | View CI/CD status checks |
| `bb pr checkout <id>` | Checkout PR branch locally |

## Development

```bash
cargo build            # Build CLI
cargo test             # Run tests
cargo run -- pr list   # Run CLI directly

```

## License

MIT
