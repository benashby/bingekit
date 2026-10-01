---
name: releasing
description: How bingekit's projects get released with release-please, and how to unstick it. Use when a change should ship, when a release pull request is red, conflicting or out of date, when merging release pull requests, when a crate needs publishing to crates.io, or when adding a project that another depends on by path.
---

# Releasing bingekit

CONTRIBUTING.md has the basics (commit types, first crates.io release). This
skill covers how the pipeline behaves in practice and the fixes for the ways it
gets stuck. Every procedure here has been run on this repository.

## How a release happens

1. A Conventional Commit scoped to one project lands on `main`, for example
   `feat(library): …`. release-please assigns commits to projects by the files
   they touch, so a commit touching two directories shows up in both
   changelogs.
2. The Release workflow (`.github/workflows/release.yml`) runs on the push and
   opens or updates one release pull request per changed project, titled
   `chore: release <project> <version>` and labelled `autorelease: pending`.
3. Merging a release pull request tags `<project>/vX.Y.Z`, creates the GitHub
   release, and runs that project's `publish-<project>` job when it publishes
   to a registry. mpv-launcher is not published (`publish = false`); it only
   gets the tag and the GitHub release.

`feat` gives a minor bump, `fix` a patch. `docs`, `ci`, `chore`, `test` and
`refactor` release nothing.

## Getting a change into a release

- A project only gets a release when a commit touches its directory. A project
  that picks up a sibling's change through a path dependency changes
  behaviour without a release, so give it its own commit (a README or
  changelog-worthy note is enough), scoped to it.
- A pull request with one commit is squash-merged. A pull request with one
  commit per project is **rebase-merged**, so each project keeps its own
  commit; a squash would merge them into one commit that lands in both
  changelogs.
- The `main` ruleset requires the `ci-ok` check. Release pull requests are
  opened with the release GitHub App's token so they get CI like any other
  pull request. A release pull request with no checks at all means the app
  token is not reaching release-please.

## Merging release pull requests

Merge them one at a time, and check each publish before the next:

```sh
gh pr merge <n> --squash
gh run list --workflow release.yml --limit 1          # then: gh run watch <id> --exit-status
gh run view <id> --json jobs -q '.jobs[]|"\(.name) \(.conclusion)"'
curl -s -A "bingekit-release-check" https://crates.io/api/v1/crates/<crate> \
  | python3 -c 'import sys,json; print(json.load(sys.stdin)["crate"]["max_version"])'
```

Publishing to crates.io can't be undone (only yanked), so a human decides
when to merge a release pull request.

## Fixes

### The other release pull request now conflicts

Every release pull request edits `.release-please-manifest.json`, so after one
merges the next usually conflicts. release-please does not rebase it. Close it
and regenerate:

```sh
gh pr close <n> --comment "Regenerating after <other release> conflicted with this one."
gh pr edit <n> --remove-label "autorelease: pending"
R=$(gh run list --workflow release.yml --limit 1 --json databaseId -q '.[0].databaseId')
J=$(gh run view $R --json jobs -q '.jobs[]|select(.name=="release-please")|.databaseId')
gh run rerun $R --job $J
```

Rerun only the `release-please` job. The publish jobs run only when that job
reports a release it just created, so they skip, but there's no reason to
rerun them. Without removing the label, release-please treats the closed
pull request as still pending and opens nothing.

### A config change doesn't show up in an open release pull request

release-please decides whether to update a release pull request by comparing
its title and body, so a change to `release-please-config.json` alone leaves
it as it was (the run logs `PR … remained the same`). Regenerate it with the
same close, unlabel and rerun steps above.

### mpv-launcher jobs fail with `--locked` on a library or episode release

```
error: the lock file …/mpv-launcher/Cargo.lock needs to be updated but --locked was passed
```

mpv-launcher depends on episode and library by path, and its `Cargo.lock`
records their versions. The episode and library packages in
`release-please-config.json` each have an `extra-files` entry that bumps their
version inside `mpv-launcher/Cargo.lock` in the same release pull request:

```json
"extra-files": [
  {
    "type": "toml",
    "path": "/mpv-launcher/Cargo.lock",
    "jsonpath": "$.package[?(@.name.value=='bingekit-library')].version"
  }
]
```

Two details that are easy to get wrong:
- The leading `/` makes the path relative to the repository root. Without it,
  the path is relative to the package's own directory.
- The filter compares `@.name.value`, not `@.name`. release-please's TOML parser
  wraps each value in an object, so `@.name=='…'` matches nothing and the run
  only logs `No entries modified`.

To check a JSONPath before pushing it, run release-please's own updater on the
real lockfile (`npm i release-please` in a scratch directory):

```js
const {GenericToml} = require("release-please/build/src/updaters/generic-toml");
const {Version} = require("release-please/build/src/version");
const fs = require("fs");
const out = new GenericToml("$.package[?(@.name.value=='bingekit-library')].version",
                            Version.parse("9.9.9")).updateContent(fs.readFileSync("Cargo.lock", "utf8"));
fs.writeFileSync("Cargo.lock.new", out);   // diff it: only that crate's version line should change
```

A project added later that another depends on by path needs the same entry
(CONTRIBUTING.md, "Adding a project").

## Reading a failed run

```sh
gh pr checks <n>                         # which jobs failed
gh run view <run-id> --log-failed | grep -E "error|Error"
gh run view <run-id> --log | grep -E "remained the same|No entries modified|Building candidate"
```
