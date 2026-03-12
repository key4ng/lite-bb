# bb CLI — Agent Reference

`bb` is a `gh`-style CLI for Bitbucket (Cloud and Server/Data Center). Use it to manage pull requests, post reviews, and interact with the Bitbucket API from the command line.

## Setup

Config lives at `~/.config/bb/config.yml`. Run `bb auth login` to authenticate, or set env vars:

```bash
# Access token (simplest)
export BB_TOKEN=your-token

# Or app password
export BB_USERNAME=your-username
export BB_APP_PASSWORD=your-app-password
```

For Bitbucket Server/DC, set `server_url` in config or `BB_SERVER_URL` env var.

## Repository Resolution

Most commands auto-detect the repository from the current git remote. To target a specific repo, use `-R`:

```bash
# Bitbucket Cloud
bb pr list -R workspace/repo-slug

# Bitbucket Server/DC (user project)
bb pr list -R "~username/repo-slug"

# Bitbucket Server/DC (project)
bb pr list -R "PROJECT_KEY/repo-slug"
```

---

## Pull Requests

### List PRs

```bash
bb pr list                          # open PRs (default)
bb pr list -s MERGED -L 10          # last 10 merged PRs
bb pr list --json                   # JSON output for parsing
bb pr list --json | jq '.[].title'  # pipe to jq
```

### View a PR

```bash
bb pr view 42                       # human-readable summary
bb pr view 42 --json                # full JSON (includes source.commit.hash)
bb pr view 42 --json | jq '.source.commit.hash'  # get latest source commit
```

JSON output includes `source.commit.hash` and `destination.commit.hash` for detecting new pushes.

### Create a PR

```bash
bb pr create -t "Add feature X" -b "Description here"
bb pr create -t "Fix bug" -H feature-branch -B main
```

### Merge a PR

```bash
bb pr merge 42
bb pr merge 42 -s squash -m "squash merge message"
```

### Other PR Operations

```bash
bb pr checkout 42                   # checkout PR branch locally
bb pr close 42                      # decline/close a PR
bb pr reopen 42                     # reopen a declined PR
bb pr edit 42 -t "New title" -b "New description" -B develop
```

---

## Code Review

### Review a PR (approve / request changes / comment)

```bash
# Approve
bb pr review 42 --approve

# Approve with comment
bb pr review 42 --approve -b "LGTM, ship it"

# Request changes with feedback
bb pr review 42 --request-changes -b "Need to handle the nil case on line 42"

# Comment only (no approve/reject)
bb pr review 42 --comment -b "Have you considered using a map here?"

# Body from file
bb pr review 42 --approve -F review.md

# Body from stdin (useful for piping LLM output)
echo "Automated review: all checks passed" | bb pr review 42 --approve -F -
```

Flags: `-a`/`--approve`, `-r`/`--request-changes`, `-c`/`--comment` (mutually exclusive). `-b`/`--body` or `-F`/`--body-file` for review text.

### Post a Comment

```bash
# PR-level comment
bb pr comment 42 -b "Overall looks good"

# Inline comment on a specific file and line
bb pr comment 42 -b "This could be None" --path src/handler.py --line 15

# Inline suggestion (use suggestion code block)
bb pr comment 42 --path src/handler.py --line 15 -b '```suggestion
    return handle_none_case(value)
```'
```

`--path` and `--line` must be used together. `--line` refers to the line number in the new (destination) file.

### List Comments on a PR

```bash
bb pr comments 42                   # human-readable
bb pr comments 42 --json            # JSON array
bb pr comments 42 --json | jq '.[].content.raw'
```

### View PR Diff

```bash
bb pr diff 42                       # unified diff output
```

### View CI/CD Checks

```bash
bb pr checks 42                     # human-readable
bb pr checks 42 --json              # JSON output
```

---

## Raw API Access

For any Bitbucket API endpoint not covered by dedicated commands:

```bash
# GET (default)
bb api repos/workspace/repo/pullrequests

# POST with JSON fields
bb api repos/workspace/repo/pullrequests -X POST -F title="My PR" -F source.branch.name=feature

# POST with raw JSON body
bb api repos/workspace/repo/pullrequests -X POST --input '{"title":"My PR"}'

# Custom headers
bb api some/endpoint -H "Accept: text/plain"

# Filter with jq
bb api repos/workspace/repo/pullrequests -q '.[].title'

# Raw output (no JSON formatting)
bb api some/endpoint --raw
```

For Server/DC, use the REST API path directly:

```bash
bb api rest/api/1.0/projects/PROJECT/repos/REPO/pull-requests
```

---

## Common Agent Workflows

### Review a PR end-to-end

```bash
# 1. Get PR metadata
pr_json=$(bb pr view 42 --json)
source_hash=$(echo "$pr_json" | jq -r '.source.commit.hash')
source_branch=$(echo "$pr_json" | jq -r '.source.branch.name')
dest_branch=$(echo "$pr_json" | jq -r '.destination.branch.name')

# 2. Get the diff
bb pr diff 42

# 3. Get existing comments
bb pr comments 42 --json

# 4. Post inline comments on issues found
bb pr comment 42 -b "Potential null dereference" --path src/main.py --line 42

# 5. Post inline suggestions
bb pr comment 42 --path src/main.py --line 42 -b '```suggestion
    if value is not None:
        process(value)
```'

# 6. Approve or request changes with summary
bb pr review 42 --request-changes -b "Found 2 issues — see inline comments"
bb pr review 42 --approve -b "All issues addressed, LGTM"
```

### Detect new commits on a PR

```bash
# Store the commit hash, compare later
current=$(bb pr view 42 --json | jq -r '.source.commit.hash')
# ... later ...
latest=$(bb pr view 42 --json | jq -r '.source.commit.hash')
[ "$current" != "$latest" ] && echo "New commits pushed"
```

### Post a multi-line review from a file

```bash
# Write review to a temp file, then post
cat <<'EOF' > /tmp/review.md
## Review Summary

- **src/auth.py:42** — Missing null check on token refresh
- **src/handler.py:15** — Race condition in concurrent requests
- **tests/** — No test coverage for the new endpoint

Overall: needs another pass before merge.
EOF

bb pr review 42 --request-changes -F /tmp/review.md
```
