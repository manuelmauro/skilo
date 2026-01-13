# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-13

### Changed

- Renamed project from `skillz` to `skilo`
- Renamed `SkillzError` to `SkiloError`
- Renamed `SKILLZ_CONFIG` environment variable to `SKILO_CONFIG`

### Added

- Table formatting support in output

## [0.1.0] - Initial Release

### Added

- `skilo new` command to scaffold new skills from templates
- `skilo lint` command to validate skills against the specification
- `skilo fmt` command to format SKILL.md files
- `skilo check` command to run all validations
- `skilo validate` command (alias for `lint --strict`)
- Support for hello-world, minimal, full, and script-based templates
- Support for Python, Bash, JavaScript, and TypeScript scripts
- Configuration via `.skilorc.toml`
- Text, JSON, and SARIF output formats
