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

# Inline comment on an added line (new file line number)
bb pr comment 42 -b "This could be None" --path src/handler.py --line 15 --line-type added

# Inline comment on a removed line (old file line number)
bb pr comment 42 -b "Why remove this?" --path src/handler.py --line 20 --line-type removed

# Inline comment on a context/unchanged line (old file line number)
bb pr comment 42 -b "Note about this" --path src/handler.py --line 10 --line-type context

# Inline suggestion (use suggestion code block)
bb pr comment 42 --path src/handler.py --line 15 --line-type added -b '```suggestion
    return handle_none_case(value)
```'
```

`--path` and `--line` must be used together. `--line-type` specifies which kind of diff line to attach to:

`--line-type` is required when using `--path` and `--line`. For new code you're reviewing, you almost always want `--line-type added`.



| `--line-type` | `--line` refers to | Use for |
|---------------|-------------------|---------|
| `added` | new file line number | `+` lines in the diff |
| `removed` | old file line number | `-` lines in the diff |
| `context` | old file line number | unchanged lines in the diff |

#### How to determine the correct `--line` number

The `--line` number must match the line's position on the correct side of the diff:

```
--- a/src/handler.py    (old file / FROM side)
+++ b/src/handler.py    (new file / TO side)
@@ -8,7 +8,9 @@
  old:8  new:8    def process(value):     ← CONTEXT: use old line 8
  old:9  new:9        validate(value)     ← CONTEXT: use old line 9
  old:10           -    return value       ← REMOVED: use old line 10
             new:10 +    if value is None: ← ADDED: use new line 10
             new:11 +        return None   ← ADDED: use new line 11
             new:12 +    return value      ← ADDED: use new line 12
  old:11 new:13       log(value)          ← CONTEXT: use old line 11
```

Examples from the diff above:

```bash
# Comment on the added "if value is None:" line
bb pr comment 42 -b "Good guard" --path src/handler.py --line 10 --line-type added

# Comment on the removed "return value" line
bb pr comment 42 -b "Why remove?" --path src/handler.py --line 10 --line-type removed

# Comment on the unchanged "def process(value):" line
bb pr comment 42 -b "Rename this?" --path src/handler.py --line 8 --line-type context
```

> **Bitbucket Server note**: For `context` lines, the server always resolves `--line` against the old file, regardless of internal `fileType`. This is a server-side behavior — `added` uses new-file numbers, `removed` and `context` both use old-file numbers.

#### Inline Suggestions

Suggestions let reviewers propose code changes that the PR author can apply with one click. Wrap the replacement code in a `` ```suggestion `` code block:

```bash
# Single-line suggestion — replaces the line at --line
bb pr comment 42 --path src/handler.py --line 10 --line-type added \
  -b '```suggestion
    if value is not None:
```'
```

The suggestion replaces the **entire line** that `--line` points to. The content inside the `` ```suggestion `` block is the replacement text.

Using the diff example from above:

```
  old:10           -    return value       ← REMOVED
             new:10 +    if value is None: ← ADDED
             new:11 +        return None   ← ADDED
             new:12 +    return value      ← ADDED
```

```bash
# Suggest changing "if value is None:" to "if not value:"
bb pr comment 42 --path src/handler.py --line 10 --line-type added \
  -b '```suggestion
    if not value:
```'

# Suggest on a context line (unchanged code)
bb pr comment 42 --path src/handler.py --line 8 --line-type context \
  -b '```suggestion
    def process(value: Optional[str]):
```'

```

> **Note**: Suggestions work with `added` and `context` line types only — you cannot suggest on a `removed` line since it's being deleted. The suggestion replaces the single line referenced by `--line`. Bitbucket renders an "Apply suggestion" button on the PR diff view. For `context` suggestions, `--line` refers to the old file line number (see [Bitbucket Server note](#how-to-determine-the-correct---line-number)).

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
bb pr comment 42 -b "Potential null dereference" --path src/main.py --line 42 --line-type added

# 5. Post inline suggestions
bb pr comment 42 --path src/main.py --line 42 --line-type added -b '```suggestion
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
