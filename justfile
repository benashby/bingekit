# Each project has its own justfile. Run a project's recipe with
# `just <project> <recipe>`, for example `just episode test`.

mod episode
mod library

default:
    @just --list --list-submodules
