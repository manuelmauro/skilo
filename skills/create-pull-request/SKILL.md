---
name: create-pull-request
description: Create GitHub pull requests with clear titles and structured descriptions. Use when the user asks to open, create, or submit a pull request or PR.
license: MIT OR Apache-2.0
---

# Create Pull Request

Create GitHub pull requests using the `gh` CLI.

## Prerequisites

- `gh` CLI installed and authenticated
- Changes committed and pushed to a remote branch

## Workflow

1. Ensure all changes are committed
2. Push the branch to the remote
3. Create the PR with `gh pr create`

## PR Title

Write a clear, human-readable title that summarizes the change. Do **not** use conventional commit format (e.g., `feat: ...`) in PR titles.

Good examples:

- `Add OAuth2 authentication support`
- `Fix redirect loop on login page`
- `Update README with new installation instructions`

Bad examples:

- `feat: ✨ add OAuth2 authentication support`
- `fix stuff`
- `WIP`

## PR Description

Use the project's PR template if one exists (`.github/pull_request_template.md`). If no template exists, structure the body as follows:

```markdown
## Description

Brief summary of what the PR does.

## Changes

- First change
- Second change

## Notes

Any additional context, trade-offs, or follow-up items.
```

## Creating the PR

```bash
# Basic
gh pr create --title "Title here" --body "Description here"

# With labels
gh pr create --title "Title here" --body "Description here" --label "enhancement"

# Draft PR
gh pr create --title "Title here" --body "Description here" --draft

# Target a specific base branch
gh pr create --title "Title here" --body "Description here" --base develop
```

## Checklist Before Creating

- [ ] Branch is up to date with base branch
- [ ] All CI checks pass locally
- [ ] PR title is clear and descriptive (no conventional commit format)
- [ ] PR description explains **what** and **why**
- [ ] Relevant reviewers or labels added if needed

## Handling Push Failures

If `git push` to the default branch is rejected due to branch protection rules, create a feature branch and push to it instead:

```bash
git checkout -b <descriptive-branch-name>
git push -u origin <descriptive-branch-name>
```
