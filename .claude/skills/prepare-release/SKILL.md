---
name: prepare-release
description: Prepare a new version release with changelog and version bumps
---

# Prepare Release

Steps to prepare a new skilo release.

## Checklist

1. All tests pass
2. No Clippy warnings
3. Update version in `Cargo.toml`
4. Update `CHANGELOG.md`
5. Update version in `README.md` CI example
6. Commit changes
7. Create git tag

## Version Bump

Update version in `Cargo.toml`:

```toml
[package]
version = "X.Y.Z"
```

## Changelog Format

Follow [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features

### Changed
- Changes to existing functionality

### Fixed
- Bug fixes
```

## Release Commands

```bash
# Verify everything passes
cargo test && cargo clippy

# Commit release changes
git add -A
git commit -m "chore: release vX.Y.Z"

# Create annotated tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# Push with tags
git push && git push --tags
```

## Publishing to crates.io

```bash
# Dry run first
cargo publish --dry-run

# Publish
cargo publish
```
