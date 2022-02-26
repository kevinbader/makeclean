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
  - [Limitations](#limitations)
  - [Hack it](#hack-it)
  - [TODO](#todo)

## Prerequisites

- Git

`makeclean` should work on Linux, MacOS and Windows. Only tested on Linux and Mac though.

## Usage

List all projects under a given path:

```bash
makeclean list --all ~/projects
```

List project using a certain build tool:

```bash
makeclean list --all --type npm
```

List only those that haven't been touched in half a year and can be safely cleaned:

```bash
makeclean list --max-age=6m
```

Clean those projects (with prompt):

```bash
makeclean clean --max-age=6m
```

Same thing but without the prompt:

```bash
makeclean clean --max-age=6m --yes
```

Dry run to see what would happen without actually deleting anything:

```bash
makeclean clean --max-age=6m --dry-run
```

If you also want to zip the projects after cleaning them:

```bash
makeclean clean --zip
```

If you _only_ want to archive them, without cleaning them first:

```bash
makeclean zip
```

Let's say you have a list of directories where you know you'll create a lot of one-off projects you don't need to keep around in a ready state:

```bash
$ cat playground.txt
~/code/rust-playground
~/code/elm-playground
~/code/flutter-playground

$ # Replacing newlines with zero-bytes is needed to process whitespace correctly without fiddling around with IFS...
$ xargs -0 -n 1 makeclean clean --archive --max-age=7d --yes < <(tr \\n \\0 <playground.txt)
```

Finally, to be able to add your own processing on top, try `makeclean list --json`.

Run `makeclean --help` to see all available options.

## Limitations

`makeclean` only supports UTF-8 encoded paths.

## Hack it

Check out the documentation on crates.io.

## TODO

- [ ] tests!
- [ ] optionally zip source code folders after removing the build dirs
- [ ] use the build tool's clean feature instead of manually removing directories; e.g. npm clean, cargo clean, mix clean, ... (maybe even check whether there's a Makefile with a clean target)
- [ ] CLI options:
  - [X] root directory
  - [X] min age
  - [ ] min size
