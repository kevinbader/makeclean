# Changelog

All changes relevant to users are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

Within node_modules directories, NPM projects are now ignored. Unless you use the directory name `node_modules` for anything else than NPM _dependencies_, `makeclean` will behave the same while cleaning, but it no longer lists NPM project dependencies as projects of their own.

**Breaking change** for developers: the `Project` struct no longer exposes its fields directly. Instead, functions now provide read-only access.

## [1.1.0] - 2022-03-31

While `--json` output still uses RFC3339 to represent the projects' modification time (`mtime`), the normal output is now hopefully more readable.

Before:

```plain
/home/user/foo (Flutter; Git; 2020-04-05 21:29:43.218548148 +00:00:00)
/home/user/bar (Cargo; Git; 2022-03-27 18:44:52.392081834 +00:00:00; 3 GiB)
/home/user/baz (Cargo; Git; 2022-03-31 17:48:51.699752798 +00:00:00; 1321 MiB)
```

Now:

```plain
/home/user/foo (Flutter; Git; a year ago)
/home/user/bar (Cargo; Git; last week; 3 GiB)
/home/user/baz (Cargo; Git; just now; 1321 MiB)
```

## [1.0.1] - 2022-03-30

Replaced [chrono] with [time] to address [RUSTSEC-2020-0159].

[chrono]: https://crates.io/crates/chrono
[time]: https://crates.io/crates/time
[RUSTSEC-2020-0159]: https://rustsec.org/advisories/RUSTSEC-2020-0159

## [1.0.0] - 2022-03-29

I'm using it regularly on Linux and MacOS. It works well and has all the features I need, so... happy v1.0! :tada: :rocket:

## [0.9.1] - 2022-03-28

Fix: When archiving, subprojects are cleaned but no longer attempted to be archived. This doesn't change the current behavior: subprojects are still included in the tar.xz file as-is and not as nested archives. But previously, the attempt to archive the subproject after the parent project produced an error, and the user needed to execute the command again to continue archiving the remaining projects.

## [0.9.0] - 2022-03-26

Support multiple directories as input. For example:

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
[Unreleased]: https://github.com/kevinbader/makeclean/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/kevinbader/makeclean/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/kevinbader/makeclean/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/kevinbader/makeclean/compare/v0.9.1...v1.0.0
[0.9.1]: https://github.com/kevinbader/makeclean/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/kevinbader/makeclean/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/kevinbader/makeclean/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/kevinbader/makeclean/compare/v0.6.0...v0.7.0
