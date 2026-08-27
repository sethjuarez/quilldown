# AGENTS.md

Guidance for AI agents and automated tooling working in this repository.

## Commits: Conventional Commits ONLY

**Every commit MUST follow the [Conventional Commits](https://www.conventionalcommits.org/) spec.** This is not optional — the release pipeline (`release-please`) parses commit messages to compute the next version and generate the changelog. Non-conforming commits are silently skipped and never appear in a release.

Format:

```
<type>[optional scope][optional !]: <description>

[optional body]

[optional footer(s)]
```

Allowed `type` values:

| Type | Use for | Version effect (pre-1.0) |
|------|---------|--------------------------|
| `feat` | A new user-facing feature | patch bump |
| `fix` | A bug fix | patch bump |
| `docs` | Documentation only | none |
| `refactor` | Code change that isn't a feature or fix | none |
| `perf` | Performance improvement | none |
| `test` | Adding or fixing tests | none |
| `build` | Build system, dependencies, packaging | none |
| `ci` | CI/workflow changes | none |
| `chore` | Maintenance, tooling, non-shipping changes | none |
| `style` | Formatting only (no logic change) | none |

Rules:

- Use a **scope** when it adds clarity: `feat(math):`, `fix(cli):`, `docs(readme):`.
- Write the description in the **imperative mood**, lowercase, no trailing period: `fix(math): stop clipping tall glyphs` (not "Fixed clipping").
- Keep the subject line ≤ ~72 characters; put detail in the body.
- **Breaking changes:** append `!` after the type/scope (`feat(cli)!: ...`) **or** add a `BREAKING CHANGE:` footer. Pre-1.0 a breaking change bumps the **minor** version (e.g. `0.2.0` → `0.3.0`).

Good:

```
feat(math): render LaTeX as native Word equations (OMML)
fix(cli): exit non-zero when the input file is missing
docs: document the --embed-svg flag
chore(deps): bump docx-rs to 0.4.22
```

Bad (these break release automation):

```
Add math rendering              # no type prefix -> skipped by release-please
Fixed a bug                     # no type prefix, past tense
update stuff                    # no type, vague
```

## Versioning & releases

- One shared version across the workspace, driven by `release-please` (`simple`
  release type). Do not hand-edit version numbers in `Cargo.toml` or the
  manifest — release-please owns them via the `# x-release-please-version`
  annotations.
- Merging a `feat:`/`fix:` to `main` opens (or updates) a release PR. Merging
  that release PR publishes to crates.io and uploads release binaries.

## Building & testing

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
```

For conversion behavior, render an example and inspect the output:

```sh
cargo run -p quilldown-cli -- examples/features/math.md -o /tmp/math.docx -v
```

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for the human-facing version of these
conventions and [`.github/skills/quilldown/SKILL.md`](./.github/skills/quilldown/SKILL.md)
for detailed usage of the tool itself.
