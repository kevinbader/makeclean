# Changelog

All changes relevant to users are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

- Support multiple directories as input. For example:

  ```bash
  makeclean -l ~/code ~/work ~/projects
  ```

## [0.8.0] - 2022-03-24

- New: support for Gradle projects
- Cargo: Cargo.toml is now already parsed when probing

## [0.7.0] - 2022-03-22

- New: support for Flutter projects
- The help message (`-h`) now contains the list of possible values for `-t`/`--type`, so you can easily see what you can filter for.

<!-- next-url -->
[Unreleased]: https://github.com/kevinbader/makeclean/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/kevinbader/makeclean/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/kevinbader/makeclean/compare/v0.6.0...v0.7.0
