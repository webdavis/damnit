# The config file carries no version and no migrations

## Context

The store has a schema version and migrations: `dam` refuses a store written by a newer version by
name, and applies every migration above the file's recorded version on open.

The config file has neither. A key that changes meaning between releases would be read under its new
meaning with no way to tell which release wrote the file.

## Decision

The config file stays unversioned before 1.0. It is a file the operator writes by hand, so a
breaking change to it is something they are told about in a release note and fix in their editor,
not something `dam` migrates behind them.

## Consequence

This is a deferral with a date on it rather than an omission. Before 1.0, a config key may change
meaning and the only protection is the release note. At 1.0 the file gets a version key and the same
refuse-a-newer-one rule the store already has.

Until then, the strict decoding is what carries the weight: every key in a remote table is refused
by name unless `dam` knows it, so a misspelled or retired key is a refusal rather than a silent
default.
