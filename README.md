# `makeclean`

[![Crates.io](https://img.shields.io/crates/v/makeclean?style=flat-square)](https://crates.io/crates/makeclean)
[![docs.rs](https://img.shields.io/docsrs/makeclean?style=flat-square)](https://docs.rs/makeclean/)

Removes generated and downloaded files from code projects to free up space.

Features:

- List, cleans and archives projects depending on how long you haven't touched them.
- Respects `.gitignore` files even outside Git repositories. Build tools often create a `.gitignore` file when initializing a new project, so this makes sure that the dependencies are not traversed even in case you have not initialized the Git repository yet.
- Supports `.ignore` files, which have the same semantics as `.gitignore` files and are supported by search tools such as ripgrep and The Silver Searcher.
- Ignores hidden directories.

Currently supports the following build tools:

- [x] Cargo
- [x] Elm
- [ ] Gradle
- [ ] Maven
- [x] Mix
- [x] NPM
- [x] Dotnet

Table of contents:

- [Installation](#installation)
- [Usage](#usage)
  - [List projects](#list-projects)
  - [Clean projects](#clean-projects)
  - [Clean + archive projects](#clean--archive-projects)
  - [Use case: automatically run for multiple project directories](#use-case-automatically-run-for-multiple-project-directories)
- [Hack it](#hack-it)
- [License](#license)

## Installation

`makeclean` should work on Linux, MacOS and Windows. Only tested on Linux and Mac though.

Install using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```bash
cargo install makeclean
```

## Usage

Run `makeclean --help` to see all available options.

### List projects

List all projects that are "stale", that is, have not been changed recently, under a given path, using `--list`/`-l`:

```bash
makeclean --list ~/projects
```

By default, a project is considered stale if there weren't any changed for at least a month. You can change this by using `--min-stale`/`-m`; for example, to consider all projects that have not been modified within the last 2 weeks:

```bash
makeclean --list --min-stale=2w ~/projects
```

Set `--min-stale` to zero to disable the check:

```bash
makeclean --list --min-stale=0 ~/projects
```

You can also filter by build tool using `--type`/`-t`:

```bash
makeclean --list --type npm ~/projects
```

### Clean projects

By default, `makeclean` looks for any projects that haven't been touched for a month, and offers to clean them:

```bash
makeclean ~/projects
```

Use `--dry-run`/`-n` to see what would happen, without actually deleting anything:

```bash
makeclean --dry-run ~/projects
```

If you run `makeclean` in a script and don't want the prompt, you can pass `--yes` to proceed automatically:

```bash
makeclean --yes ~/projects
```

### Clean + archive projects

If you also want to archive the projects after cleaning them up, pass `--archive`. For example, the following command would replace the contents of `~/projects/foo` with `~/projects/foo.tar.xz`, after cleaning it:

```bash
makeclean --archive ~/projects/foo
```

> Note that while `--archive` also considers cleaned projects, it still respects `--min-stale`. If makeclean doesn't find your project but you think it should, try again with the environment variable `RUST_LOG` set to `trace`, e.g., `RUST_LOG=trace makeclean --archive ~/projects/foo`. You should see a hint as to why the project was not considered. If the logs don't tell you what's going on, please consider [creating a GitHub issue](https://github.com/kevinbader/makeclean/issues/new).

To restore the project, use `tar` (which is probably already installed on your system):

```bash
cd ~/projects/foo
tar -xaf foo.tar.xz && rm foo.tar.xz
```

### Use case: automatically run for multiple project directories

Let's say you have a list of directories where you know you'll create a lot of one-off projects you don't need to keep around in a ready state. You can use the following command to automically process them:

```bash
$ cat playground.txt
~/code/rust-playground
~/code/elm-playground
~/code/flutter-playground

$ # Replacing newlines with zero-bytes is needed to process whitespace correctly without fiddling around with IFS...
$ xargs -0 -n 1 makeclean --min-stale=7d --yes < <(tr \\n \\0 <playground.txt)
```

## Hack it

Check out the documentation on crates.io. PRs welcome!

## License

MIT. Any contributions are assumed MIT-licensed as well.
