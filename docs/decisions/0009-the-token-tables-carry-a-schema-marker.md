# The token tables carry a schema marker

## Context

Decision 0006 keeps the config file unversioned before 1.0. Tokens change that for one part of the
file: a setup tool may deploy a config written for a newer `dam`, and an older `dam` that misread a
token's terms would grant more than the owner wrote.

## Decision

The file may hold `schema = 1` at the top level. It governs only the `[token.<name>]` tables. A
`schema` other than 1 disables every token, and the rest of the file is read as before. Adding a
permission never changes the schema; only a change to how a token table is read does.

Inside a token table, an unknown permission is skipped, since skipping only narrows. An unknown key
disables that token, since `dam` cannot enforce a restriction or condition it does not know. A
`limits` table disables its token for the same reason until `dam` enforces limits.

## Consequence

The token tables are read leniently while every other section stays strict: a bad token never stops
`dam` from running for the owner. `dam status` names each skipped setting and disabled token.
