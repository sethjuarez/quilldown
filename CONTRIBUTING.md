# Contributing to quilldown

Thanks for helping improve quilldown! A few conventions keep the project
releasable and the history readable.

## Commit messages: Conventional Commits (required)

This repository uses **[Conventional Commits](https://www.conventionalcommits.org/)**
for **every** commit. We don't just prefer it — our release automation
(`release-please`) reads commit messages to decide the next version number and
to build the changelog. **A commit without a valid type prefix is ignored by the
release tooling** and won't show up in any release notes.

### Format

```
<type>[optional scope][optional !]: <description>

[optional body]

[optional BREAKING CHANGE: footer]
```

### Types

- **`feat:`** — a new user-facing feature
- **`fix:`** — a bug fix
- **`docs:`** — documentation only
- **`refactor:`** — a code change that neither fixes a bug nor adds a feature
- **`perf:`** — a performance improvement
- **`test:`** — adding or correcting tests
- **`build:`** — build system, dependencies, or packaging
- **`ci:`** — CI / GitHub Actions changes
- **`chore:`** — maintenance that doesn't ship to users
- **`style:`** — formatting only, no behavior change

### Guidelines

- Use a **scope** in parentheses when it helps: `feat(math):`, `fix(cli):`.
- Write the description in the **imperative mood**, lowercase, and with no
  trailing period — e.g. `fix(math): stop clipping tall glyphs`.
- Keep the subject line short (≤ ~72 chars); explain the "why" in the body.
- **Breaking changes:** add `!` after the type/scope (e.g. `feat(cli)!: ...`)
  or include a `BREAKING CHANGE:` footer. Now that the project is 1.0+, a
  breaking change bumps the **major** version.

### Examples

```
feat(math): render LaTeX as native Word equations (OMML)
fix(cli): exit non-zero when the input file is missing
docs: document the --embed-svg flag
chore(deps): bump docx-rs to 0.4.22
```

Please **don't** write commits like `Add math`, `Fixed a bug`, or `update
stuff` — they break the release pipeline.

## Before you push

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
```

## Releases

You don't need to bump versions by hand. `release-please` maintains one shared
workspace version and opens a release PR as `feat:`/`fix:` commits land on
`main`. Merging that release PR publishes the crates to crates.io and attaches
prebuilt binaries to the GitHub Release.
