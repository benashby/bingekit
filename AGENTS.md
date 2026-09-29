# Notes for coding agents

Read CONTRIBUTING.md first. The rules there apply to you, and CI enforces the commit
rules.

- Never add AI attribution anywhere: no `Co-Authored-By` trailer for a model or tool, no
  "Generated with" line, no robot emoji.
- Never write a "how to test" or "test plan" section in a commit message or pull
  request.
- Humanize every piece of prose you write before committing it: commit messages,
  Markdown and code comments. If the `humanizer` skill is available, run the text
  through it. If not, follow the Writing section of CONTRIBUTING.md.
- Each top-level directory is an independent project with its own lockfile. Build and
  test inside the project you change. Do not add a repository-wide Cargo workspace.
- Commits are Conventional Commits scoped to one project, for example
  `fix(episode): …`.
