# Contributing

## Layout

Each top-level directory is an independent project. Work inside the project you are
changing, with that language's own tools. Projects do not share a lockfile or a build.
A project that uses another one depends on it by path and still keeps its own lockfile,
and its CI filter lists the other project's directory so a change there tests it too.

## Commits

Commit messages use [Conventional Commits](https://www.conventionalcommits.org/) with the
project directory as the scope:

```
feat(episode): read NCOP and NCED as extras

Creditless openings were parsed as episode 1 and sorted in with the main run.
```

The type decides the release. `fix` makes a patch release, `feat` a minor one, and a `!`
after the type (or a `BREAKING CHANGE:` footer) a major one. `docs`, `test`, `refactor`,
`build`, `ci` and `chore` do not release anything. Release tooling works out which
project a commit belongs to from the files it changes, so keep each commit to one
project.

Write the subject in the imperative, lower case, without a full stop. Use the body to say
why the change was needed. The diff already shows what changed.

Two things are not allowed, in commits or in pull requests, and CI rejects them:

- Attribution to AI tools, such as `Co-Authored-By: Claude`, "Generated with…" lines, or
  the robot emoji.
- A "how to test", "test plan" or "testing instructions" section.

## Writing

All prose in this repository reads as written by a person: commit messages, pull
requests, documentation and code comments. Say the thing plainly and specifically. Avoid
the patterns Wikipedia lists in
[Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing),
especially:

- "not X but Y" contrasts
- one-line closing lines that restate the point
- em and en dashes
- lists of three that exist for rhythm
- inflated words such as "robust", "seamless" or "crucial"
- bold labels on every list item

## Releasing

Pushing to `main` updates one release pull request per changed project. Merging it tags
`<project>/vX.Y.Z`, creates the GitHub release, and publishes to the project's registry
from `release.yml`. release-please runs as the bingekit-release GitHub App, so its pull
requests get CI like any other. The app's client ID is the `RELEASE_APP_CLIENT_ID`
variable and its key the `RELEASE_APP_PRIVATE_KEY` secret.

A crate's first crates.io release needs an API token, because Trusted Publishing can
only be set up for a crate that already exists. Store the token as `CARGO_REGISTRY_TOKEN`
in the `crates-io` environment and merge the release pull request. Then register
`release.yml` as the crate's trusted publisher and delete the secret. The publish job uses
the secret while it exists and Trusted Publishing otherwise.

## Adding a project

Until a scaffold script exists, a new project needs:

1. its directory, with a README, a copy of LICENSE, its language's own policy files
   (for Rust, a `deny.toml`), and a `nix.nix` if Nix should build it;
2. a `mod` line in the root `justfile`;
3. an entry in `release-please-config.json` and `.release-please-manifest.json`. Don't
   add a CHANGELOG: release-please writes it in the first release pull request, and a
   file it didn't write ends up appended below its entries;
4. a filter and jobs in `.github/workflows/ci.yml`, with the jobs added to `ci-ok`;
5. an entry in `.github/dependabot.yml`, a publish job in `release.yml` if it
   publishes to a registry, and an `area:<project>` label;
6. a row in the table in `README.md`.
