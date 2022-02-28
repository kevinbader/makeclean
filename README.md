# `makeclean`

Removes generated and downloaded files from code projects to free up space.

What does this do? Its main features:

- find projects
  - ignore projects in dependency and build directories
  - ignore projects that are ignored by version control
  - => finds all directories not ignored by VCS and checks them for projects
- clean projects
  - run the build tool's "clean" command
  - OR: delete the dependency and build directories
- archive projects (BONUS)
  - after cleaning, archive the project, create checksum file
- provide an interactive CLI mode

- [`makeclean`](#makeclean)
  - [Prerequisites](#prerequisites)
  - [Usage](#usage)
    - [List projects](#list-projects)
    - [Clean projects](#clean-projects)
    - [Clean + archive projects](#clean--archive-projects)
    - [Use case: automatically run for multiple project directories](#use-case-automatically-run-for-multiple-project-directories)
  - [Limitations](#limitations)
  - [Hack it](#hack-it)
  - [TODO](#todo)

## Prerequisites

- Git

`makeclean` should work on Linux, MacOS and Windows. Only tested on Linux and Mac though.

## Usage

> Run `makeclean --help` to see all available options.

### List projects

List all projects under a given path using `--list`/`-l`:

```bash
makeclean --list ~/projects
```

List all projects under a given path that have not changed in the last month using `--min-age`/`-m`:

```bash
makeclean --list --min-age=1m ~/projects
```

Set it to zero to disable the check:

```bash
makeclean --list --min-age=0 ~/projects
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

> Note that while `--archive` also considers cleaned projects, it still respects `--min-age`. If makeclean doesn't find your project but you think it should, try again with the environment variable `RUST_LOG` set to `trace`, e.g., `RUST_LOG=trace makeclean --archive ~/projects/foo`. You should see a hint as to why the project was not considered. If the logs don't tell you what's going on, please consider creating a GitHub issue.

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
$ xargs -0 -n 1 makeclean --min-age=7d --yes < <(tr \\n \\0 <playground.txt)
```

## Limitations

`makeclean` only supports UTF-8 encoded paths.

## Hack it

Check out the documentation on crates.io.

## TODO

- [X] tests!
- [X] optionally tar source code folders after removing the build dirs
- [ ] use the build tool's clean feature instead of manually removing directories; e.g. npm clean, cargo clean, mix clean, ... (maybe even check whether there's a Makefile with a clean target)
- [ ] CLI options:
  - [X] root directory
  - [X] min age
