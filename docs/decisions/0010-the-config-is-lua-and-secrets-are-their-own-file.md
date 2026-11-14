# The config is Lua and secrets are their own file

## Context

The config was one TOML file. It held settings, remote credentials and token tables together. A
person or a dotfiles template writes the settings, while damnit's own commands write tokens. One
file means a template that rewrites the settings can overwrite what a command wrote. It also means
the config is never safe to show, since any part of it may be a secret. TOML has only values, so a
credential that a person wants computed needs its own `_command` or `_env` form.

## Decision

The config is two Lua files, each returning a table. `config.lua` holds the settings: `owner`,
`done`, `remote`, `category` and `filter`. It never holds a secret, so it is always safe to show.
`secrets.lua`, in the same folder, holds every secret: each remote's credentials and the token
tables. A credential is a string, or a function that dam calls only when its remote is used.

`schema` moves to `secrets.lua`, since it governs only the token tables. This supersedes the TOML
parts of decisions 0006 and 0009 without changing their rules. Every section of `config.lua` and
the top level of `secrets.lua` stay strict, and the token tables stay lenient.

## Consequence

Lua gives plain values for facts and functions where a person wants behavior. A template can own
`config.lua`. In `secrets.lua`, damnit's commands edit the token table and keep everything else as
written, so a person may write the rest, credential functions included. The TOML reader is
removed in the same body of work, with no migration: an existing `config.toml` is rewritten by hand.
`docs/examples/` holds one of each file, and `docs/schemas/` describes the tables they return.
