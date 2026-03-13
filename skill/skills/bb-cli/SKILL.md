---
name: bb-cli
description: "Use the lite-bb (`bb`) CLI for Bitbucket operations instead of `gh`. Trigger this skill whenever the user is working with a Bitbucket-hosted repository and needs to create, review, merge, list, or manage pull requests, repositories, or API calls — even if they say 'create a PR' without mentioning Bitbucket explicitly. Also trigger when the user mentions Bitbucket, bb CLI, or asks about Bitbucket workflows. If the repo's git remote points to bitbucket.org or a Bitbucket Server instance, this skill applies."
---

# bb — Bitbucket CLI for Claude Code

`bb` is a `gh`-style CLI for Bitbucket Cloud and Server/Data Center. When working in a Bitbucket-hosted repo, always use `bb` instead of `gh`.

## Detecting Bitbucket repos

Before running any PR or repo command, check the git remote:

```bash
git remote -v
```

If the remote URL contains `bitbucket.org` or a known Bitbucket Server domain, use `bb` for all operations. Do NOT use `gh` — it won't work with Bitbucket.

## Setup

If `bb` is not installed, suggest:

```bash
pip install lite-bb
# or
uv pip install lite-bb
```

Then authenticate — always suggest token-based auth first since it's simpler:

```bash
bb auth login
```

This prompts the user to choose between:
- **Access Token** (recommended): a single token value — simplest setup
- **App Password**: username + password pair

Check auth status with:

```bash
bb auth status
```

Environment variables override config: `BB_TOKEN` for access tokens, or `BB_USERNAME` + `BB_APP_PASSWORD` for app passwords. For Bitbucket Server/DC, also set `BB_SERVER_URL`.

## Pull Requests

All PR commands auto-detect workspace/repo from git remotes. Override with `-R WORKSPACE/REPO`.

### List PRs

```bash
bb pr list                        # open PRs (default)
bb pr list -s MERGED -L 10        # last 10 merged PRs
bb pr list --json                 # JSON output for scripting
```

Flags: `-s/--state` (OPEN|MERGED|DECLINED|SUPERSEDED), `-L/--limit` (default 30), `--json`, `-R/--repo`

### View a PR

```bash
bb pr view 42                     # human-readable summary
bb pr view 42 --json              # full JSON with metadata
```

### Create a PR

```bash
bb pr create -t "Add feature X" -b "Description here"
bb pr create -t "Fix bug" -H feature-branch -B main
```

Flags: `-t/--title` (required), `-b/--body`, `-H/--head` (source branch, defaults to current), `-B/--base` (destination, defaults to repo default branch)

### Edit a PR

```bash
bb pr edit 42 -t "Updated title"
bb pr edit 42 -b "New description" -B develop
```

At least one of `--title`, `--body`, or `--base` is required.

### Merge a PR

```bash
bb pr merge 42
bb pr merge 42 -s squash -m "squash: feature X"
```

Flags: `-s/--strategy` (merge_commit|squash|fast_forward), `-m/--message`

### Close / Reopen

```bash
bb pr close 42          # decline the PR
bb pr reopen 42         # reopen a declined PR
```

### Checkout PR branch

```bash
bb pr checkout 42       # fetches and checks out the PR's source branch
```

### View PR diff

```bash
bb pr diff 42           # unified diff output
```

## Code Review

### Approve / Request Changes / Comment on a PR

```bash
bb pr review 42 --approve
bb pr review 42 --approve -b "LGTM, ship it"
bb pr review 42 --request-changes -b "Need to handle the edge case on line 42"
bb pr review 42 --comment -b "Have you considered using a map here?"
bb pr review 42 --approve -F review.md        # body from file
echo "Looks good" | bb pr review 42 --approve -F -   # body from stdin
```

Exactly one of `--approve`, `--request-changes`, or `--comment` is required. `--comment` requires `--body` or `--body-file`.

### Post comments on a PR

```bash
# PR-level comment
bb pr comment 42 -b "Overall looks good"

# Inline comment on an added line
bb pr comment 42 -b "This could be None" --path src/handler.py --line 15 --line-type added

# Inline comment on a removed line
bb pr comment 42 -b "Why remove this?" --path src/handler.py --line 20 --line-type removed

# Inline comment on a context (unchanged) line
bb pr comment 42 -b "Note about this" --path src/handler.py --line 10 --line-type context

# Code suggestion (use ```suggestion fenced block)
bb pr comment 42 --path src/handler.py --line 15 --line-type added -b '```suggestion
    return handle_none_case(value)
```'
```

Inline comment rules:
- `--path` and `--line` must both be present for inline comments
- `--line-type` is **required** with inline comments: `added`, `removed`, or `context`
- For `added` lines, use the new file's line number
- For `removed` or `context` lines, use the old file's line number

### List comments

```bash
bb pr comments 42               # human-readable
bb pr comments 42 --json        # JSON array
```

### Check CI status

```bash
bb pr checks 42                 # shows ✓/✗/○/■ status
bb pr checks 42 --json
```

## Repositories

### List repos

```bash
bb repo list                          # repos in default workspace
bb repo list my-workspace             # specific workspace
bb repo list --visibility private     # filter by visibility
bb repo list --json
```

### View repo

```bash
bb repo view                          # current repo
bb repo view myworkspace/my-repo      # specific repo
bb repo view --web                    # open in browser
bb repo view --json
```

### Clone repo

```bash
bb repo clone myworkspace/my-repo             # clone into ./my-repo
bb repo clone myworkspace/my-repo ./local     # custom directory
bb repo clone myworkspace/my-repo . -- --depth 1   # shallow clone
```

Prefers SSH over HTTPS when available.

### Create repo

```bash
bb repo create -n my-new-repo -d "My project"     # private by default
bb repo create -n my-new-repo --public             # public repo
bb repo create -n my-new-repo --public --clone     # create and clone locally
```

## Code Search

```bash
bb search code "fn main"                          # search whole workspace
bb search code "TODO" -R myworkspace/myrepo       # search specific repo
bb search code "import requests" --extension py   # filter by extension
bb search code "apiKey" --json
```

Flags: `-R/--repo`, `-L/--limit` (default 30), `--extension`, `--filename`, `--json`

## Raw API Access

`bb api` is a raw passthrough to the Bitbucket API — use it for anything not covered by the higher-level commands.

```bash
# GET (default)
bb api repositories/myworkspace/myrepo

# GET with jq filter
bb api repositories/myworkspace/myrepo/pullrequests -q '[.values[].title]'

# POST with JSON fields
bb api repositories/myworkspace/myrepo/pullrequests -X POST \
  -F title="My PR" -F description="Fixes the bug"

# POST with raw JSON body
bb api rest/search/latest/search -X POST \
  --input '{"query":"deepseek","entities":{"code":{"start":0,"limit":3}}}' \
  -q '.code.values[].file'

# Custom headers and raw output
bb api some/endpoint -H 'X-Custom: value' --raw
```

Flags: `-X/--method`, `-F/--field` (repeatable, auto-typed), `--input` (raw JSON), `-H/--header` (repeatable), `-q/--jq` (requires jq in PATH), `--raw`

Base URLs:
- Cloud: `https://api.bitbucket.org/2.0`
- Server/DC: configured server root (supports `/rest/api/1.0/`, `/rest/search/latest/`, etc.)

## Config

Config lives at `~/.config/bb/config.yml` (respects `XDG_CONFIG_HOME` and `BB_CONFIG_DIR`):

```yaml
# Access token auth (simplest)
token: "your-access-token"

# OR app password auth
username: "your-username"
app_password: "your-app-password"

# For Bitbucket Server/DC
server_url: "https://bitbucket.company.com"

# Defaults (optional)
workspace: "myworkspace"
default_repo: "my-repo"
```

Credential priority: env vars > config file.

## Repo format

- **Cloud**: `workspace/repo-slug`
- **Server/DC**: `PROJECT_KEY/repo-slug` or `~username/repo-slug` (personal repos)

## Key differences from gh

| gh | bb | Notes |
|----|-----|-------|
| `gh pr create` | `bb pr create` | Same flags |
| `gh pr merge --squash` | `bb pr merge -s squash` | Strategy flag differs |
| `gh pr review --approve` | `bb pr review --approve` | Same |
| `gh pr close` | `bb pr close` | bb uses "decline" internally |
| `gh api` | `bb api` | Same concept, Bitbucket endpoints |
| `gh repo clone` | `bb repo clone` | Same |
| N/A | `bb search code` | Bitbucket code search |
| States: open/closed/merged | States: OPEN/MERGED/DECLINED/SUPERSEDED | Bitbucket has SUPERSEDED |
