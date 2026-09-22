# dam-remote-gcal, read-only: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `dam-remote-gcal`, the Google Calendar remote the spec names, as a read-only helper:
a one-time sign-in that mints the refresh token the spec's config reads, a pull that brings the
operator's configured calendars into dam's store and refuses every push by name, and a `dam agenda`
listing a scheduler reads busy times from.

**Architecture:** One new package, `dam-remote-gcal`, holds every line of Google code: the consent
walk (binary `dam-gcal-sign-in`) and the protocol helper (binary `dam-remote-gcal`), both thin mains
over one library. Four small changes to dam's core carry what that helper needs and the spec does not
provide yet: the helper learns its remote's name and address, an attendee records whether it is the
calendar's own, a pull can report an event cancelled by id, and `dam agenda` reads events as
intervals with a busy reading. Each core change is its own pull request and its own proposed spec
amendment.

**Tech Stack:** Rust 2024 edition, stable toolchain. `ureq` 3 with rustls, `serde` and `serde_json`,
`jiff` 0.2, `getrandom` 0.3, and one new workspace dependency, `sha2` 0.11, for the PKCE challenge.

**Spec:** `docs/superpowers/specs/2026-09-18-damnit-design.md`. This plan is the "its own plan" the
spec's crate layout promises for `dam-remote-gcal`. Where the spec is silent or would have to change,
the plan says so in [Proposed spec amendments](#proposed-spec-amendments) and applies nothing to the
spec until the operator approves it.

## Global constraints

Copied from the spec, the clean-code Rust standard, the v1 plan and the operator's rules for this
work. Every task's requirements include these.

- Helpers are tested against a loopback double of Google's API, never the live service.
- Every `dam` binary run in a test sets `HOME`, `XDG_CONFIG_HOME` and `XDG_DATA_HOME` to a temporary directory.
- Never read `~/.config/dam/`, `~/.config/gogcli/`, or any rendered secret file; never run the real `gog`; never run dam against a real account or the network.
- OAuth credentials and tokens are secrets: never in argv, a child's environment, a log line or an error string. Mirror the discipline pns's `GoogleCalendar` type documents.
- No code in this workspace depends on another repository.
- Nothing dam ships names pns: no code, comment, test name, README line, spec text or output. This
  plan names it only where it points at the consent walk being copied. The sign-in code is a copy,
  never a path or git dependency, and no crate is shared with that repository.
- Every helper binary run in a test sets the same three variables as a `dam` run, so no test can
  reach a real home directory whichever binary it drives.
- Read-only means read-only: nothing `dam-remote-gcal` does can create, change or delete a Google
  Calendar event. The helper sends no request on a push, and the sign-in asks for the read-only scope,
  so Google refuses a write even if code tried one.
- No em-dashes anywhere: code, comments, docs, commit messages, pull request bodies.
- Conventional Commits, one logical change per commit, `SKIP_AI_COMMIT=1` in the environment, no AI
  co-author trailer, no generated-with footer, never `--no-verify`.
- `trash`, never `rm`, including scratch you made.
- Comments say what the code does or why it is that way, never what was rejected or considered.
- New behavior is written test-first, without exception: failing test, run it, see it fail for the
  intended reason, then the code.
- Every test finishes within one second. Bind `support::guard` in every integration test, as the
  Todoist helper's tests do.
- File size: 200 implementation lines and 300 total are the targets; 250 implementation or 400 total
  requires decomposition; no handwritten `.rs` file exceeds 500 total lines, tests included. Count
  with the command in `~/.agents/skills/clean-code-rust/SKILL.md`; `just size` is the gate.
- Every `main.rs` and every `src/bin/*.rs` stays under 150 lines.
- Domain crate: `std` and `jiff` only. A time zone reaches it as an argument, never read there.
- Private by default. `pub` only for intentional crate APIs, curated in `lib.rs`.
- No `unwrap` or `expect` on untrusted input. No panic on ordinary external failure.
- Synchronous everywhere. No async runtime.
- Gates before every commit: `just gates` (fmt, clippy with `-D warnings`, build, test, doc, size).
- Every crate pins `channel = "stable"` in its own `rust-toolchain.toml`.
- A crate's tests use fixtures and doubles it owns: `dam-remote-gcal` carries its own copy of the
  loopback double and the speed guard, as `dam-remote-todoist` does, so it still builds the day it
  moves to a repository of its own.

## Review Focus

The five inputs this work will meet that the spec never names, most likely first. Each line's test
is written into the task named after it.

1. **A meeting the operator declined.** Google keeps it on the calendar, opaque and confirmed. A
   person expects it not to count as busy. Pinned in Task 13
   (`an_event_the_calendars_own_attendee_declined_holds_no_time`).
2. **A meeting deleted in Google Calendar after dam pulled it.** Google reports it as a cancelled
   item carrying only its id, or not at all once it purges it. A person expects the listing to stop
   calling that hour busy. Pinned in Tasks 6, 10 and 11 (`cancelled`, and events that vanish from
   the window).
3. **An all-day event on a daylight-saving day.** The day is 23 or 25 hours long. A person expects
   the interval to run midnight to midnight, not a fixed 86,400 seconds. Pinned in Task 13
   (`an_all_day_span_on_a_spring_forward_day_is_twenty_three_hours`).
4. **A calendar id with `#` and `@` in it**, such as a holiday calendar. A person expects it read, not
   a 404 from an unencoded path. Pinned in Task 9
   (`a_calendar_id_is_one_percent_encoded_path_segment`).
5. **A refresh token Google revoked.** A person expects to be told to sign in again, and never to see
   the token or the client secret in the message. Pinned in Task 8
   (`a_revoked_refresh_token_says_to_sign_in_again_and_quotes_nothing`).

---

## Pull requests

Six pull requests. The first four are independent of one another and can be reviewed in parallel;
the fifth needs all four merged; the sixth needs the third.

| PR | Branch | Tasks | Needs |
|---|---|---|---|
| 1 | `feat/gcal-sign-in` | 1, 2, 3: package, consent walk, `dam-gcal-sign-in` | nothing |
| 2 | `feat/helper-remote-name-and-address` | 4: helper invocation | nothing |
| 3 | `feat/attendee-self` | 5: the calendar's own attendee | nothing |
| 4 | `feat/pull-cancelled` | 6: a pull reports a cancellation by id | nothing |
| 5 | `feat/gcal-read-only-pull` | 7 to 12: the read-only helper | 1, 2, 3, 4 |
| 6 | `feat/agenda` | 13, 14, 15: `dam agenda` | 3 |

Each pull request applies the text of its own spec amendment as its last commit, and only once the
operator has approved that amendment; an amendment still pending stops that step and is raised with
the operator rather than applied or skipped. PR 1 applies A4's "Signing in" paragraph, PR 2 A1,
PR 3 A2, PR 4 A3, PR 5 the rest of A4, and PR 6 A5.

Every branch starts from `main` in its own worktree:

```bash
herdr worktree create --cwd /Users/stephen/workspaces/Ivy/webdavis/damnit --branch <branch> --no-focus
```

## File structure

```
crates/
  dam-protocol/
    src/invocation.rs                 credential_variable, moved here from dam-adapters     (PR 2)
    src/messages.rs                   PullResponse.cancelled                                 (PR 4)
    src/wire.rs                       WireAttendee.is_self, `self` on the wire               (PR 3)
    docs/specs/protocol.md            invocation, `cancelled`                                (PR 2, 4)
    fixtures/pull-cancelled.response.json                                                    (PR 4)
  dam-domain/
    src/object/event.rs               Attendee.is_self; Event::span, Event::holds_time       (PR 3, 6)
    src/object/event/tests.rs                                                                (PR 6)
    src/when/mod.rs                   When::instant                                          (PR 6)
  dam-application/
    src/config.rs                     RemoteConfig::address                                  (PR 2)
    src/remote.rs                     PullOutcome.cancelled                                  (PR 4)
    src/use_cases/pull/cancelled.rs   cancelled ids become incoming cancelled events         (PR 4)
    src/use_cases/agenda.rs           Window, Scheduled, agenda()                            (PR 6)
  dam-adapters/
    src/helper_process.rs             `<remote> <address>` on the helper's command line      (PR 2)
    src/wire/{encode,decode,mod}.rs   is_self both ways; cancelled into PullOutcome           (PR 3, 4)
  dam-cli/
    src/args/reading.rs               AgendaArgs                                             (PR 6)
    src/commands/agenda.rs            run_agenda, the agenda document, the human line        (PR 6)
    tests/agenda.rs                   the real binary, the document's contract               (PR 6)
  dam-remote-gcal/                                                                            (PR 1, 5)
    Cargo.toml                        package dam-remote-gcal; bins dam-gcal-sign-in, dam-remote-gcal
    rust-toolchain.toml
    src/lib.rs                        curated exports
    src/secret.rs                     Secret: redacted Debug, no Display
    src/endpoints.rs                  Endpoints: production, and the DAM_GCAL_BASE_URL loopback seam
    src/http.rs                       the one agent configuration, the bounded read
    src/encoding.rs                   form bodies, percent encoding both ways, base64url
    src/oauth_error.rs                the RFC 6749 error word an answer names, and nothing else
    src/sign_in.rs                    SignIn, Client, SignInError, mint
    src/sign_in/pkce.rs               verifier, S256 challenge, random tokens
    src/sign_in/redirect.rs           the authorization URL, the one redirect and its code
    src/sign_in/exchange.rs           the authorization_code grant
    src/bin/dam-gcal-sign-in.rs       the operator's one-time walk
    src/address.rs                    the calendars the remote's address names                (PR 5)
    src/credentials.rs                Credentials from DAM_<REMOTE>_<NAME>                     (PR 5)
    src/capabilities.rs               kinds, fields, credentials                               (PR 5)
    src/access.rs                     the refresh_token grant                                  (PR 5)
    src/api_error.rs                  ApiError                                                 (PR 5)
    src/calendar_api.rs               CalendarApi::events, Window, paging                      (PR 5)
    src/calendar_api/resources.rs     Google's event resource, as much of it as dam reads      (PR 5)
    src/map.rs                        a Google event onto a wire event, or a cancelled id      (PR 5)
    src/map/time.rs                   start and end: text dam reads, epoch seconds             (PR 5)
    src/memory.rs                     what the last pull reported, the sync token              (PR 5)
    src/pull.rs                       the window, every calendar, cancellations                (PR 5)
    src/push.rs                       every mutation refused, by name                          (PR 5)
    src/main.rs                       dam-remote-gcal: `<remote> <address>`, the protocol loop  (PR 5)
    tests/loopback/mod.rs             the Google double: routes, queued replies, recorded requests
    tests/support/mod.rs              the speed guard, copied from dam-remote-todoist
    tests/sign_in_walk.rs, tests/sign_in_binary.rs, tests/api.rs, tests/pull.rs, tests/protocol.rs
```

## Shared vocabulary

| Name | Crate | Created in |
|---|---|---|
| `Secret` | dam-remote-gcal | Task 1 |
| `Endpoints`, `BASE_URL_VARIABLE` | dam-remote-gcal | Task 1 |
| `SignIn`, `Client`, `SignInError` | dam-remote-gcal | Task 2 |
| `credential_variable` (moved) | dam-protocol | Task 4 |
| `RemoteConfig::address` | dam-application | Task 4 |
| `Attendee::is_self`, `WireAttendee::is_self` | dam-domain, dam-protocol | Task 5 |
| `PullResponse::cancelled`, `PullOutcome::cancelled` | dam-protocol, dam-application | Task 6 |
| `calendars`, `Credentials`, `capabilities` | dam-remote-gcal | Task 7 |
| `access_token`, `ApiError` | dam-remote-gcal | Task 8 |
| `CalendarApi`, `Window`, `Listing` | dam-remote-gcal | Task 9 |
| `Mapped`, `MapError`, `remote_id`, `pull`, `PullError` | dam-remote-gcal | Task 10 |
| `Memory` | dam-remote-gcal | Task 11 |
| `refuse`, `READ_ONLY` | dam-remote-gcal | Task 12 |
| `When::instant`, `Event::span`, `Event::holds_time` | dam-domain | Task 13 |
| `Window`, `Scheduled`, `agenda` | dam-application | Task 14 |

Google facts this plan rests on, read on 2026-09-22 from
`https://developers.google.com/workspace/calendar/api/v3/reference/events/list`,
`https://developers.google.com/workspace/calendar/api/v3/reference/events`,
`https://developers.google.com/workspace/calendar/api/guides/sync` and
`https://developers.google.com/identity/protocols/oauth2/native-app`:

- `GET https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events`. `timeMin` is an
  exclusive lower bound on an event's end, `timeMax` an exclusive upper bound on its start.
  `singleEvents=true` expands recurring events into instances. `showDeleted=true` includes cancelled
  events. `maxResults` defaults to 250 and tops out at 2500. `nextPageToken` pages. The response
  carries `summary` (the calendar's title) and `timeZone` (the calendar's zone).
- `https://www.googleapis.com/auth/calendar.events.readonly` is among the scopes `events.list`
  accepts.
- `status` is `confirmed`, `tentative` or `cancelled`. A cancelled exception is only guaranteed to
  carry `id`, `recurringEventId` and `originalStartTime`; a deleted event only `id`.
- `transparency` is `opaque` or `transparent`; `visibility` is `default`, `public`, `private`,
  `confidential`; `eventType` is `birthday`, `default`, `focusTime`, `fromGmail`, `outOfOffice`,
  `workingLocation`. `attendees[].self` is "whether this entry represents the calendar on which this
  copy of the event appears"; `attendees[].responseStatus` is `needsAction`, `declined`, `tentative`,
  `accepted`.
- The installed-app flow: authorization at `https://accounts.google.com/o/oauth2/v2/auth`, tokens at
  `https://oauth2.googleapis.com/token`, a loopback redirect `http://127.0.0.1:<port>`, PKCE with
  `S256`, and `error=access_denied` on the redirect when the person refuses.

---

### Task 1: The package, its secret type and its endpoints

PR 1.

**Files:**
- Modify: `Cargo.toml` (member `crates/dam-remote-gcal`; workspace dependency `sha2 = "0.11"`)
- Create: `crates/dam-remote-gcal/Cargo.toml`, `crates/dam-remote-gcal/rust-toolchain.toml`
- Create: `crates/dam-remote-gcal/src/lib.rs`, `src/secret.rs`, `src/endpoints.rs`

**Interfaces:**
- Produces:

```rust
// secret.rs
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);                     // Debug prints Secret(<redacted>); no Display
impl Secret { pub fn expose(&self) -> &str; }
impl From<String> for Secret; impl From<&str> for Secret;

// endpoints.rs
pub const BASE_URL_VARIABLE: &str = "DAM_GCAL_BASE_URL";
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoints { pub authorization: String, pub token: String, pub calendar: String }
impl Endpoints {
    pub fn production() -> Endpoints;
    pub fn loopback(base: &str) -> Result<Endpoints, String>;   // refuses anything but http://127.0.0.1:<port> or http://localhost:<port>
    pub fn from_env() -> Result<Endpoints, String>;             // production unless DAM_GCAL_BASE_URL is set
}
```

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-remote-gcal/src/secret.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_never_debug_prints_its_value() {
        let secret = Secret::from("1//0gSUPERSECRETREFRESH");
        assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
        assert_eq!(secret.expose(), "1//0gSUPERSECRETREFRESH");
    }
}
```

```rust
// crates/dam-remote-gcal/src/endpoints.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_names_googles_three_hosts() {
        let e = Endpoints::production();
        assert_eq!(e.authorization, "https://accounts.google.com/o/oauth2/v2/auth");
        assert_eq!(e.token, "https://oauth2.googleapis.com/token");
        assert_eq!(e.calendar, "https://www.googleapis.com/calendar/v3");
    }

    #[test]
    fn a_loopback_base_serves_all_three_under_one_port() {
        let e = Endpoints::loopback("http://127.0.0.1:8080/").unwrap();
        assert_eq!(e.authorization, "http://127.0.0.1:8080/o/oauth2/v2/auth");
        assert_eq!(e.token, "http://127.0.0.1:8080/token");
        assert_eq!(e.calendar, "http://127.0.0.1:8080/calendar/v3");
        assert!(Endpoints::loopback("http://localhost:9").is_ok());
    }

    /// The seam carries a refresh token and a client secret to whatever it
    /// names, so it names nothing but this machine.
    #[test]
    fn the_seam_refuses_every_address_but_loopback() {
        for refused in [
            "https://oauth2.googleapis.com",
            "http://evil.test:8080",
            "http://127.0.0.1.evil.test:8080",
            "http://user@127.0.0.1:8080",
            "http://localhost:8080@evil.test",
            "http://127.0.0.1",
            "http://127.0.0.1:",
            "https://127.0.0.1:8080",
            "",
        ] {
            let err = Endpoints::loopback(refused).unwrap_err();
            assert!(err.contains(BASE_URL_VARIABLE), "{refused}: {err}");
        }
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-remote-gcal`
Expected: error, `package ID specification dam-remote-gcal did not match any packages` (the package
does not exist yet). After Step 3's manifests and before its code, compile errors naming `Secret` and
`Endpoints`.

- [ ] **Step 3: Implement**

```toml
# crates/dam-remote-gcal/Cargo.toml
[package]
name = "dam-remote-gcal"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "the Google Calendar remote helper for dam, read-only"

[lib]
path = "src/lib.rs"

[dependencies]
dam-protocol = { path = "../dam-protocol" }
getrandom.workspace = true
serde.workspace = true
serde_json.workspace = true
sha2.workspace = true
ureq.workspace = true
```

```toml
# crates/dam-remote-gcal/rust-toolchain.toml
[toolchain]
channel = "stable"
```

In the root `Cargo.toml`, add `"crates/dam-remote-gcal"` to `members` after
`"crates/dam-remote-todoist"`, and `sha2 = "0.11"` to `[workspace.dependencies]`.

```rust
// crates/dam-remote-gcal/src/lib.rs
//! The Google Calendar remote helper for dam. Read-only: it pulls events and
//! never creates, changes or deletes one.

pub mod endpoints;
pub mod secret;

pub use endpoints::{BASE_URL_VARIABLE, Endpoints};
pub use secret::Secret;
```

`secret.rs` is `crates/dam-remote-todoist/src/api/token.rs` copied and renamed: `ApiToken` becomes
`Secret`, the doc comment says "A credential or token", and `Debug` writes `Secret(<redacted>)`.

```rust
// crates/dam-remote-gcal/src/endpoints.rs
//! Where the three Google calls go: the consent page, the token endpoint and
//! the Calendar API.

pub const BASE_URL_VARIABLE: &str = "DAM_GCAL_BASE_URL";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoints {
    pub authorization: String,
    pub token: String,
    pub calendar: String,
}

impl Endpoints {
    pub fn production() -> Endpoints {
        Endpoints {
            authorization: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token: "https://oauth2.googleapis.com/token".into(),
            calendar: "https://www.googleapis.com/calendar/v3".into(),
        }
    }

    /// All three under one test server, which is what the loopback double serves.
    pub fn loopback(base: &str) -> Result<Endpoints, String> {
        let base = checked_base(base)?;
        Ok(Endpoints {
            authorization: format!("{base}/o/oauth2/v2/auth"),
            token: format!("{base}/token"),
            calendar: format!("{base}/calendar/v3"),
        })
    }

    pub fn from_env() -> Result<Endpoints, String> {
        match std::env::var(BASE_URL_VARIABLE) {
            Ok(base) => Endpoints::loopback(&base),
            Err(_) => Ok(Endpoints::production()),
        }
    }
}
```

`checked_base` is `checked_base` from `crates/dam-remote-todoist/src/api.rs` copied, with the
refusal naming `DAM_GCAL_BASE_URL` instead of `DAM_TODOIST_BASE_URL`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: 4 passed; gates clean.

- [ ] **Step 5: Commit**

```bash
SKIP_AI_COMMIT=1 git add Cargo.toml Cargo.lock crates/dam-remote-gcal
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): add the package with its secret type and loopback-only endpoints"
```

---

### Task 2: The consent walk

PR 1. Copied from the consent walk in the dotfiles repository,
`pns/crates/pns-adapters/src/calendar/google/consent.rs` and `token.rs` (read, never depended on),
with four differences: randomness comes from `getrandom` rather than `/dev/urandom`, base64url is the
twenty lines below rather than a dependency, the scope is `calendar.events.readonly`, and a refusal
of the exchange names Google's HTTP status and its RFC 6749 error word.

**Files:**
- Create: `crates/dam-remote-gcal/src/http.rs`, `src/encoding.rs`, `src/oauth_error.rs`
- Create: `src/sign_in.rs`, `src/sign_in/pkce.rs`, `src/sign_in/redirect.rs`, `src/sign_in/exchange.rs`
- Create: `tests/loopback/mod.rs`, `tests/support/mod.rs`, `tests/sign_in_walk.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `Secret`, `Endpoints` (Task 1).
- Produces:

```rust
// http.rs
pub fn agent() -> ureq::Agent;   // pub so lib.rs can re-export it (Task 8); the module stays pub(crate)
pub(crate) struct Answer { pub(crate) status: u16, pub(crate) body: String }   // Task 9 adds retry_after
pub(crate) enum ReadError { Transport(String), TooLarge { limit: u64 } }        // Display names which
pub(crate) fn read(response: ureq::http::Response<ureq::Body>, max: u64) -> Result<Answer, ReadError>;
pub(crate) const MAX_ANSWER: u64 = dam_protocol::MAX_LINE;

// encoding.rs
pub(crate) fn percent_encoded(text: &str) -> String;       // RFC 3986 unreserved set passes, every other byte %XX
pub(crate) fn percent_decoded(text: &str) -> String;       // %XX and `+` as space; a malformed escape stays literal
pub(crate) fn form(fields: &[(&str, &str)]) -> String;     // name=value pairs joined by &, both sides encoded
pub(crate) fn base64url(bytes: &[u8]) -> String;           // RFC 4648 section 5, unpadded

// oauth_error.rs
pub(crate) fn oauth_error_code(body: &str) -> Option<&'static str>;   // one of RFC 6749 section 5.2's six words, or None

// sign_in.rs
pub struct Client { pub id: String, pub secret: Secret }
#[derive(Debug, PartialEq, Eq)]
pub enum SignInError { Listener, Random, Redirect, State, Denied(&'static str), NoCode, Exchange { status: Option<u16>, code: Option<&'static str> }, NoRefreshToken }
pub struct SignIn { /* agent, endpoints */ }
impl SignIn {
    pub fn new(endpoints: Endpoints) -> SignIn;
    pub fn mint(&self, client: &Client, announce: &mut dyn FnMut(&str)) -> Result<Secret, SignInError>;
}
```

- [ ] **Step 1: Write the failing unit tests**

```rust
// crates/dam-remote-gcal/src/encoding.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4648 section 10's vectors, in the URL-safe alphabet with no padding.
    #[test]
    fn base64url_matches_the_rfc_vectors() {
        for (input, expected) in [
            ("", ""), ("f", "Zg"), ("fo", "Zm8"), ("foo", "Zm9v"),
            ("foob", "Zm9vYg"), ("fooba", "Zm9vYmE"), ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64url(input.as_bytes()), expected, "{input}");
        }
        assert_eq!(base64url(&[0xfb, 0xff]), "-_8");
    }

    #[test]
    fn percent_encoding_passes_the_unreserved_set_and_escapes_every_other_byte() {
        assert_eq!(percent_encoded("aZ09-._~"), "aZ09-._~");
        assert_eq!(percent_encoded("a b&c=d/é#@"), "a%20b%26c%3Dd%2F%C3%A9%23%40");
    }

    #[test]
    fn percent_decoding_reads_escapes_and_plus_and_keeps_a_broken_escape() {
        assert_eq!(percent_decoded("a%20b+c%2F"), "a b c/");
        assert_eq!(percent_decoded("100%zz%4"), "100%zz%4");
        assert_eq!(percent_decoded(&percent_encoded("4/0Ab_x&y=z")), "4/0Ab_x&y=z");
    }

    /// A credential carrying `&` or `=` must not compose a field nobody wrote.
    #[test]
    fn a_form_body_encodes_names_and_values() {
        assert_eq!(form(&[("a", "1&b=2"), ("c d", "")]), "a=1%26b%3D2&c%20d=");
    }
}
```

```rust
// crates/dam-remote-gcal/src/sign_in/pkce.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 7636 appendix B, the one published S256 example.
    #[test]
    fn the_challenge_is_the_rfc_example() {
        assert_eq!(
            challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    /// Thirty-two random bytes encode to forty-three characters, the shortest
    /// verifier RFC 7636 admits, and two draws differ.
    #[test]
    fn a_random_token_is_forty_three_url_safe_characters() {
        let a = random_token().unwrap();
        let b = random_token().unwrap();
        assert_eq!(a.len(), 43);
        assert!(a.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'));
        assert_ne!(a, b);
    }
}
```

```rust
// crates/dam-remote-gcal/src/sign_in/redirect.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Endpoints;

    /// Pinned byte for byte: a scope, a parameter or an encoding that moves is
    /// a consent screen asking for something else.
    #[test]
    fn the_authorization_url_asks_for_read_only_events_offline_with_pkce() {
        let url = authorization_url(
            &Endpoints::production(),
            "123.apps.googleusercontent.com",
            "http://127.0.0.1:5555",
            "CHALLENGE",
            "STATE",
        );
        assert_eq!(
            url,
            "https://accounts.google.com/o/oauth2/v2/auth?client_id=123.apps.googleusercontent.com\
             &redirect_uri=http%3A%2F%2F127.0.0.1%3A5555&response_type=code\
             &scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcalendar.events.readonly\
             &code_challenge=CHALLENGE&code_challenge_method=S256&state=STATE\
             &access_type=offline&prompt=consent"
        );
    }

    #[test]
    fn the_code_is_read_once_the_state_agrees() {
        assert_eq!(code_of("GET /?state=S1&code=4%2F0Ab HTTP/1.1", "S1"), Ok("4/0Ab".to_string()));
    }

    #[test]
    fn a_redirect_for_another_run_is_refused() {
        assert_eq!(code_of("GET /?state=OTHER&code=c HTTP/1.1", "S1"), Err(SignInError::State));
        assert_eq!(code_of("GET /?code=c HTTP/1.1", "S1"), Err(SignInError::State));
    }

    /// Google's own word for a refusal is a fixed vocabulary, so it is named;
    /// anything else in `error` is not quoted.
    #[test]
    fn a_refusal_in_the_browser_names_googles_word_and_nothing_else() {
        assert_eq!(
            code_of("GET /?error=access_denied&state=S1 HTTP/1.1", "S1"),
            Err(SignInError::Denied("access_denied"))
        );
        assert_eq!(
            code_of("GET /?error=%3Cscript%3E&state=S1 HTTP/1.1", "S1"),
            Err(SignInError::Denied("an error Google did not name"))
        );
    }

    #[test]
    fn a_redirect_with_no_code_is_refused() {
        assert_eq!(code_of("GET /?state=S1 HTTP/1.1", "S1"), Err(SignInError::NoCode));
        assert_eq!(code_of("GET /?state=S1&code= HTTP/1.1", "S1"), Err(SignInError::NoCode));
    }
}
```

```rust
// crates/dam-remote-gcal/src/oauth_error.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_word_is_named_and_the_rest_of_the_body_is_not() {
        let body = r#"{"error":"invalid_grant","error_description":"Bad Request 1//0gSECRET"}"#;
        assert_eq!(oauth_error_code(body), Some("invalid_grant"));
        assert_eq!(oauth_error_code(r#"{"error":"1//0gSECRET"}"#), None);
        assert_eq!(oauth_error_code("not json"), None);
    }
}
```

- [ ] **Step 2: Write the failing walk test and its double**

`tests/support/mod.rs` is `crates/dam-remote-todoist/tests/support/mod.rs` copied unchanged.

The double is the Todoist helper's `tests/loopback/mod.rs` extended three ways: it records the query
string and the raw body text (the token endpoint takes a form, not JSON), a route answers from a
queue (so a test can script pages), and a reply can carry headers (for `Retry-After`).

```rust
// crates/dam-remote-gcal/tests/loopback/mod.rs
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub query: String,
    pub body: String,
    pub authorization: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

impl Reply {
    pub fn json(status: u16, body: serde_json::Value) -> Reply {
        Reply { status, body: body.to_string(), headers: vec![] }
    }
    pub fn with_header(mut self, name: &str, value: &str) -> Reply {
        self.headers.push((name.into(), value.into()));
        self
    }
}

pub struct Loopback {
    pub base: String,
    pub seen: Arc<Mutex<Vec<Seen>>>,
}

impl Loopback {
    pub fn seen(&self) -> Vec<Seen> {
        self.seen.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

/// `routes` maps "METHOD /path", the query left off, to the replies it gives
/// in order; the last one repeats. An unmatched request gets 404.
pub fn serve(routes: HashMap<&'static str, Vec<Reply>>) -> Loopback {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("bind: {e}"));
    let base = format!("http://{}", listener.local_addr().unwrap_or_else(|e| panic!("{e}")));
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let mut served: HashMap<String, usize> = HashMap::new();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let method = parts.next().unwrap_or("").to_string();
            let target = parts.next().unwrap_or("").to_string();
            let (path, query) = target.split_once('?').map_or((target.clone(), String::new()), |(p, q)| (p.to_string(), q.to_string()));
            let mut length = 0usize;
            let mut authorization = None;
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).is_err() || header.trim().is_empty() {
                    break;
                }
                let lower = header.to_ascii_lowercase();
                if let Some(v) = lower.strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap_or(0);
                }
                if lower.starts_with("authorization:") {
                    authorization = header.split_once(':').map(|(_, v)| v.trim().to_string());
                }
            }
            let mut raw = vec![0u8; length];
            let _ = reader.read_exact(&mut raw);
            let body = String::from_utf8_lossy(&raw).into_owned();
            if let Ok(mut l) = log.lock() {
                l.push(Seen { method: method.clone(), path: path.clone(), query, body, authorization });
            }
            let key = format!("{method} {path}");
            let reply = routes.get(key.as_str()).and_then(|queue| {
                let n = served.entry(key.clone()).or_insert(0);
                let reply = queue.get(*n).or_else(|| queue.last()).cloned();
                *n += 1;
                reply
            });
            let reply = reply.unwrap_or_else(|| Reply::json(404, serde_json::json!({"error": "no route"})));
            let mut head = format!("HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n", reply.status, reply.body.len());
            for (name, value) in &reply.headers {
                head.push_str(&format!("{name}: {value}\r\n"));
            }
            let _ = reader.get_mut().write_all(format!("{head}\r\n{}", reply.body).as_bytes());
        }
    });
    Loopback { base, seen }
}
```

```rust
// crates/dam-remote-gcal/tests/sign_in_walk.rs
mod loopback;
mod support;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use dam_remote_gcal::{Client, Endpoints, SignIn, SignInError};
use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn client() -> Client {
    Client { id: "123.apps.googleusercontent.com".into(), secret: CLIENT_SECRET.into() }
}

/// The value of one query parameter in the announced URL.
fn param(url: &str, name: &str) -> String {
    let query = url.split_once('?').map(|(_, q)| q).unwrap_or_default();
    let raw = query.split('&').find_map(|p| p.strip_prefix(&format!("{name}="))).unwrap_or_default();
    raw.replace("%3A", ":").replace("%2F", "/")
}

/// Plays the browser: opens the redirect with the given query, returns the page.
fn browse(url: &str, query: impl Fn(&str) -> String) -> std::thread::JoinHandle<String> {
    let redirect = param(url, "redirect_uri");
    let state = param(url, "state");
    let address = redirect.trim_start_matches("http://").to_string();
    let line = format!("GET /?{} HTTP/1.1\r\nHost: {address}\r\n\r\n", query(&state));
    std::thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(line.as_bytes()).unwrap();
        let mut page = String::new();
        let _ = stream.read_to_string(&mut page);
        page
    })
}

#[test]
fn a_granted_consent_is_exchanged_for_the_refresh_token() {
    let _guard = support::guard("a_granted_consent_is_exchanged_for_the_refresh_token");
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![Reply::json(200, serde_json::json!({
        "access_token": "ya29.ACCESS", "expires_in": 3599, "refresh_token": REFRESH,
        "scope": "https://www.googleapis.com/auth/calendar.events.readonly", "token_type": "Bearer"
    }))]);
    let google = loopback::serve(routes);
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut browser = None;
    let token = sign_in
        .mint(&client(), &mut |url| {
            browser = Some(browse(url, |state| format!("state={state}&code=4%2F0AbCODE&scope=x")));
        })
        .unwrap();
    assert_eq!(token.expose(), REFRESH);
    let page = browser.unwrap().join().unwrap();
    assert!(page.contains("200"), "{page}");
    let seen = google.seen();
    assert_eq!(seen.len(), 1);
    let body = &seen[0].body;
    for field in ["grant_type=authorization_code", "code=4%2F0AbCODE", "client_id=123.apps.googleusercontent.com", "client_secret=GOCSPX-SUPERSECRETCLIENT", "code_verifier="] {
        assert!(body.contains(field), "{field} missing from the exchange");
    }
    assert!(body.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A"), "{body}");
}

#[test]
fn a_refused_exchange_names_the_status_and_googles_word_and_quotes_nothing() {
    let _guard = support::guard("a_refused_exchange_names_the_status_and_googles_word_and_quotes_nothing");
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![Reply::json(401, serde_json::json!({
        "error": "invalid_client", "error_description": format!("echo {CLIENT_SECRET}")
    }))]);
    let google = loopback::serve(routes);
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut browser = None;
    let err = sign_in
        .mint(&client(), &mut |url| {
            browser = Some(browse(url, |state| format!("state={state}&code=c")));
        })
        .unwrap_err();
    let _ = browser.unwrap().join();
    assert_eq!(err, SignInError::Exchange { status: Some(401), code: Some("invalid_client") });
    let said = err.to_string();
    assert!(said.contains("401") && said.contains("invalid_client"), "{said}");
    assert!(!said.contains(CLIENT_SECRET), "{said}");
}

#[test]
fn an_exchange_answering_no_refresh_token_is_refused_by_name() {
    let _guard = support::guard("an_exchange_answering_no_refresh_token_is_refused_by_name");
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![Reply::json(200, serde_json::json!({"access_token": "ya29.A", "expires_in": 3599}))]);
    let google = loopback::serve(routes);
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut browser = None;
    let err = sign_in
        .mint(&client(), &mut |url| {
            browser = Some(browse(url, |state| format!("state={state}&code=c")));
        })
        .unwrap_err();
    let _ = browser.unwrap().join();
    assert_eq!(err, SignInError::NoRefreshToken);
}

#[test]
fn a_consent_refused_in_the_browser_sends_no_exchange() {
    let _guard = support::guard("a_consent_refused_in_the_browser_sends_no_exchange");
    let google = loopback::serve(HashMap::new());
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut browser = None;
    let err = sign_in
        .mint(&client(), &mut |url| {
            browser = Some(browse(url, |state| format!("error=access_denied&state={state}")));
        })
        .unwrap_err();
    let page = browser.unwrap().join().unwrap();
    assert_eq!(err, SignInError::Denied("access_denied"));
    assert!(page.contains("refused"), "{page}");
    assert!(google.seen().is_empty());
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p dam-remote-gcal`
Expected: compile errors naming `encoding`, `SignIn`, `Client`, `SignInError`.

- [ ] **Step 4: Implement**

Add `pub(crate) mod encoding; pub(crate) mod http; pub(crate) mod oauth_error; pub mod sign_in;` and
`pub use sign_in::{Client, SignIn, SignInError};` to `lib.rs`.

```rust
// crates/dam-remote-gcal/src/http.rs
//! One agent configuration and one bounded read, shared by every Google call.

use std::fmt;
use std::time::Duration;

/// The most of any one answer read. dam receives a pull as one protocol line,
/// so an answer past that line could never be delivered.
pub(crate) const MAX_ANSWER: u64 = dam_protocol::MAX_LINE;

/// Thirty seconds a call, redirects refused (a redirect is how a credential
/// reaches a host nobody meant), and every status read as an answer.
pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .into()
}

pub(crate) struct Answer {
    pub(crate) status: u16,
    pub(crate) body: String,
}

pub(crate) enum ReadError {
    Transport(String),
    TooLarge { limit: u64 },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Transport(s) => write!(f, "the connection failed: {s}"),
            ReadError::TooLarge { limit } => write!(f, "the answer is past the {limit} byte ceiling"),
        }
    }
}

pub(crate) fn read(response: ureq::http::Response<ureq::Body>, max: u64) -> Result<Answer, ReadError> {
    let status = response.status().as_u16();
    let body = response
        .into_body()
        .into_with_config()
        .limit(max)
        .read_to_string()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(limit) => ReadError::TooLarge { limit },
            other => ReadError::Transport(other.to_string()),
        })?;
    Ok(Answer { status, body })
}
```

```rust
// crates/dam-remote-gcal/src/encoding.rs
//! Percent encoding both ways, form bodies, and unpadded base64url.

pub(crate) fn percent_encoded(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(byte as char),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

pub(crate) fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escaped = (bytes[i] == b'%')
            .then(|| text.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (bytes[i], escaped) {
            (_, Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (b'+', None) => {
                out.push(b' ');
                i += 1;
            }
            (byte, None) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn form(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encoded(name), percent_encoded(value)))
        .collect::<Vec<_>>()
        .join("&")
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub(crate) fn base64url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}
```

```rust
// crates/dam-remote-gcal/src/oauth_error.rs
//! The one word of a refused OAuth answer that may be repeated: RFC 6749
//! section 5.2's error codes. The rest of the body is where a refused
//! exchange echoes the credentials back, so it is never quoted.

const CODES: [&str; 6] = [
    "invalid_request", "invalid_client", "invalid_grant",
    "unauthorized_client", "unsupported_grant_type", "invalid_scope",
];

pub(crate) fn oauth_error_code(body: &str) -> Option<&'static str> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let stated = value.get("error")?.as_str()?;
    CODES.into_iter().find(|code| *code == stated)
}
```

```rust
// crates/dam-remote-gcal/src/sign_in/pkce.rs
//! RFC 7636: the verifier this run keeps, and the S256 challenge it shows.

use sha2::Digest;

use super::SignInError;
use crate::encoding::base64url;

pub(super) fn challenge(verifier: &str) -> String {
    base64url(sha2::Sha256::digest(verifier.as_bytes()).as_slice())
}

/// Thirty-two bytes from the operating system, as the forty-three characters
/// a verifier and a state are each written in.
pub(super) fn random_token() -> Result<String, SignInError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| SignInError::Random)?;
    Ok(base64url(&bytes))
}
```

```rust
// crates/dam-remote-gcal/src/sign_in/redirect.rs
//! The consent URL, and the single browser redirect that answers it.

use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use super::SignInError;
use crate::Endpoints;
use crate::encoding::{form, percent_decoded};

/// The one scope asked for. Read-only, so Google refuses a write whatever
/// code tried one.
pub(super) const SCOPE: &str = "https://www.googleapis.com/auth/calendar.events.readonly";
/// How long the redirect may take once the browser has connected.
const REDIRECT_READ_DEADLINE: Duration = Duration::from_secs(30);
/// The most of the browser's request read; a redirect line is hundreds of bytes.
const REDIRECT_READ_MAX: u64 = 8 * 1024;
/// Google's words for a refusal on the redirect.
const DENIALS: [&str; 5] = ["access_denied", "invalid_scope", "invalid_request", "unauthorized_client", "server_error"];

pub(super) fn authorization_url(endpoints: &Endpoints, client_id: &str, redirect_uri: &str, challenge: &str, state: &str) -> String {
    let query = form(&[
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", SCOPE),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
        ("access_type", "offline"),
        ("prompt", "consent"),
    ]);
    format!("{}?{query}", endpoints.authorization)
}

/// One connection and one only: the operator is sitting at this walk, so the
/// listener waits for the browser they were told to open, and the browser is
/// told the outcome either way.
pub(super) fn await_code(listener: &TcpListener, state: &str) -> Result<String, SignInError> {
    let (mut stream, _) = listener.accept().map_err(|_| SignInError::Redirect)?;
    stream.set_read_timeout(Some(REDIRECT_READ_DEADLINE)).map_err(|_| SignInError::Redirect)?;
    let mut line = String::new();
    std::io::BufReader::new((&stream).take(REDIRECT_READ_MAX))
        .read_line(&mut line)
        .map_err(|_| SignInError::Redirect)?;
    let answered = code_of(&line, state);
    let _ = stream.write_all(page(answered.is_ok()).as_bytes());
    answered
}

pub(super) fn code_of(request_line: &str, state: &str) -> Result<String, SignInError> {
    let query = request_line
        .split_whitespace()
        .nth(1)
        .and_then(|target| target.split_once('?'))
        .map(|(_, q)| q)
        .unwrap_or_default();
    let stated = |name: &str| {
        query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == name)
            .map(|(_, value)| percent_decoded(value))
    };
    if stated("state").as_deref() != Some(state) {
        return Err(SignInError::State);
    }
    if let Some(error) = stated("error") {
        let word = DENIALS.into_iter().find(|d| *d == error).unwrap_or("an error Google did not name");
        return Err(SignInError::Denied(word));
    }
    stated("code").filter(|code| !code.is_empty()).ok_or(SignInError::NoCode)
}

fn page(granted: bool) -> String {
    let body = if granted {
        "dam has the consent. Close this window and read the terminal."
    } else {
        "dam refused this redirect. Close this window and read the terminal."
    };
    format!(
        "HTTP/1.1 200 OK\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}
```

The state check comes before the error check on purpose: a redirect that does not carry this run's
state is answered as not ours, whatever else it says.

```rust
// crates/dam-remote-gcal/src/sign_in/exchange.rs
//! The authorization_code grant: the code and the verifier in, the refresh token out.

use super::{Client, SignInError};
use crate::Secret;
use crate::encoding::form;
use crate::http::{MAX_ANSWER, read};
use crate::oauth_error::oauth_error_code;

pub(super) fn exchange(agent: &ureq::Agent, token_endpoint: &str, client: &Client, code: &str, verifier: &str, redirect_uri: &str) -> Result<Secret, SignInError> {
    let body = form(&[
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", &client.id),
        ("client_secret", client.secret.expose()),
        ("code_verifier", verifier),
        ("redirect_uri", redirect_uri),
    ]);
    let refused = |status| SignInError::Exchange { status, code: None };
    let response = agent
        .post(token_endpoint)
        .header("content-type", "application/x-www-form-urlencoded")
        .send(body.as_str())
        .map_err(|_| refused(None))?;
    let answer = read(response, MAX_ANSWER).map_err(|_| refused(None))?;
    if !(200..300).contains(&answer.status) {
        return Err(SignInError::Exchange { status: Some(answer.status), code: oauth_error_code(&answer.body) });
    }
    let value: serde_json::Value = serde_json::from_str(&answer.body).map_err(|_| refused(Some(answer.status)))?;
    value
        .get("refresh_token")
        .and_then(serde_json::Value::as_str)
        .filter(|t| !t.is_empty())
        .map(Secret::from)
        .ok_or(SignInError::NoRefreshToken)
}
```

```rust
// crates/dam-remote-gcal/src/sign_in.rs
//! The one-time consent walk that mints the refresh token a gcal remote reads.
//!
//! Google's installed-application flow: a loopback redirect on an ephemeral
//! port, PKCE with S256, and `access_type=offline` with `prompt=consent`, which
//! is what makes the exchange answer with a refresh token. The credentials
//! travel in the exchange's request body and nowhere else, and every refusal
//! is a fixed sentence naming the step.

mod exchange;
mod pkce;
mod redirect;

use std::fmt;
use std::net::TcpListener;

use crate::{Endpoints, Secret};

/// The address the redirect comes back to. Loopback only: the code is a
/// credential, and a listener on any other interface offers it to the network.
const REDIRECT_HOST: &str = "127.0.0.1";

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub secret: Secret,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SignInError {
    Listener,
    Random,
    Redirect,
    State,
    Denied(&'static str),
    NoCode,
    Exchange { status: Option<u16>, code: Option<&'static str> },
    NoRefreshToken,
}

impl fmt::Display for SignInError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignInError::Listener => f.write_str("no loopback port could be opened for the redirect, so nothing was asked for"),
            SignInError::Random => f.write_str("no random bytes could be read, so nothing was asked for"),
            SignInError::Redirect => f.write_str("the browser redirect did not arrive, so no code was exchanged"),
            SignInError::State => f.write_str("the redirect did not carry this run's state, so it was not answered"),
            SignInError::Denied(word) => write!(f, "the consent was refused in the browser ({word})"),
            SignInError::NoCode => f.write_str("the redirect carried no authorization code, so nothing was exchanged"),
            SignInError::Exchange { status: None, .. } => f.write_str("the code exchange did not reach Google or its answer could not be read"),
            SignInError::Exchange { status: Some(s), code: None } => write!(f, "Google refused the code exchange (HTTP {s})"),
            SignInError::Exchange { status: Some(s), code: Some(c) } => write!(f, "Google refused the code exchange (HTTP {s}, {c})"),
            SignInError::NoRefreshToken => f.write_str("Google answered the exchange with no refresh token"),
        }
    }
}

impl std::error::Error for SignInError {}

pub struct SignIn {
    agent: ureq::Agent,
    endpoints: Endpoints,
}

impl SignIn {
    pub fn new(endpoints: Endpoints) -> SignIn {
        SignIn { agent: crate::http::agent(), endpoints }
    }

    /// One consent: the URL handed to `announce` for the operator to open, the
    /// single redirect read off a loopback port, and the code exchanged. The
    /// verifier and the state are minted here, so no caller can reuse one.
    pub fn mint(&self, client: &Client, announce: &mut dyn FnMut(&str)) -> Result<Secret, SignInError> {
        let verifier = pkce::random_token()?;
        let state = pkce::random_token()?;
        let listener = TcpListener::bind((REDIRECT_HOST, 0)).map_err(|_| SignInError::Listener)?;
        let port = listener.local_addr().map_err(|_| SignInError::Listener)?.port();
        let redirect_uri = format!("http://{REDIRECT_HOST}:{port}");
        announce(&redirect::authorization_url(&self.endpoints, &client.id, &redirect_uri, &pkce::challenge(&verifier), &state));
        let code = redirect::await_code(&listener, &state)?;
        exchange::exchange(&self.agent, &self.endpoints.token, client, &code, &verifier, &redirect_uri)
    }
}
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: every test passes, each well inside a second; gates clean.

- [ ] **Step 6: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): the consent walk that mints a read-only refresh token"
```

---

### Task 3: The `dam-gcal-sign-in` binary

PR 1. The walk lives in the helper's package, not in `dam`, because it is Google code and the spec
keeps every line of Google code out of dam's core ("`dam` core contains no Todoist code and no Google
code"). It is a binary of its own rather than a word on `dam-remote-gcal`, because dam runs
`dam-remote-gcal` and nothing else runs it: neither program parses the other's arguments, and one
`cargo install` ships both.

**Files:**
- Create: `crates/dam-remote-gcal/src/bin/dam-gcal-sign-in.rs`
- Create: `crates/dam-remote-gcal/src/sign_in/command_line.rs` (the two checks the binary makes before any network)
- Modify: `crates/dam-remote-gcal/src/sign_in.rs` (`mod command_line;`, `pub use command_line::{client_id, client_secret};`)
- Create: `crates/dam-remote-gcal/tests/sign_in_binary.rs`
- Modify: `crates/dam-remote-gcal/Cargo.toml` (`[[bin]] name = "dam-gcal-sign-in"`)
- Modify: `README.md` (a "Google Calendar" section: install, the Google client, the sign-in)

**Interfaces:**
- Consumes: `SignIn`, `Client`, `Endpoints::from_env`, `Secret` (Tasks 1 and 2).
- Produces: `sign_in::client_id(&[String]) -> Result<String, String>` and
  `sign_in::client_secret(impl Read, is_terminal: bool) -> Result<Secret, String>`, and the binary
  `dam-gcal-sign-in --client-id <id>`, the client secret on standard input. Standard
  output carries the refresh token and nothing else; standard error carries the URL and the
  instructions. Exit 0 minted, 1 the walk failed, 2 the command line or standard input was wrong.

- [ ] **Step 1: Write the failing tests**

Unit tests for the two decisions the binary makes before any network. They live in the library so
the binary stays a thin main well under 150 lines:

```rust
// bottom of crates/dam-remote-gcal/src/sign_in/command_line.rs
#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn the_client_id_is_the_one_flag() {
        assert_eq!(client_id(&words("--client-id 123.apps")), Ok("123.apps".to_string()));
        assert!(client_id(&words("")).unwrap_err().contains("--client-id"));
        assert!(client_id(&words("--client-id")).unwrap_err().contains("--client-id"));
        assert!(client_id(&words("--client-id a --verbose")).unwrap_err().contains("--verbose"));
    }

    /// A secret in argv is readable by every process on the machine, so the
    /// flag is refused by name rather than failing as a typo would.
    #[test]
    fn a_client_secret_on_the_command_line_is_refused_by_name() {
        for line in ["--client-id a --client-secret s", "--client-id a --client-secret=s"] {
            let said = client_id(&words(line)).unwrap_err();
            assert!(said.contains("standard input"), "{said}");
            assert!(!said.contains("=s"), "{said}");
        }
    }

    /// Typed at a terminal the secret echoes into scrollback, so a terminal
    /// on standard input is refused and the message says how to pipe it.
    #[test]
    fn the_secret_is_read_from_a_pipe_and_refused_from_a_terminal() {
        assert_eq!(client_secret("GOCSPX-abc\n".as_bytes(), false).unwrap().expose(), "GOCSPX-abc");
        assert!(client_secret("GOCSPX-abc\n".as_bytes(), true).unwrap_err().contains("pipe"));
        assert!(client_secret(" \n".as_bytes(), false).unwrap_err().contains("no client secret"));
    }
}
```

The whole binary, through the real process:

```rust
// crates/dam-remote-gcal/tests/sign_in_binary.rs
mod loopback;
mod support;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};

use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn sign_in(base: &str, home: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_dam-gcal-sign-in"));
    command
        .args(["--client-id", "123.apps.googleusercontent.com"])
        .env("DAM_GCAL_BASE_URL", base)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_HOME", home.join("data"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

#[test]
fn the_refresh_token_alone_reaches_standard_output() {
    let _guard = support::guard("the_refresh_token_alone_reaches_standard_output");
    let home = tempfile::tempdir().unwrap();
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![Reply::json(200, serde_json::json!({"refresh_token": REFRESH, "access_token": "ya29.A"}))]);
    let google = loopback::serve(routes);
    let mut child = sign_in(&google.base, home.path()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(format!("{CLIENT_SECRET}\n").as_bytes()).unwrap();
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let url = loop {
        let mut line = String::new();
        assert!(stderr.read_line(&mut line).unwrap() > 0, "the URL never arrived");
        if let Some(url) = line.trim().strip_prefix(&format!("{}/o/oauth2/v2/auth?", google.base)) {
            break url.to_string();
        }
    };
    let param = |name: &str| url.split('&').find_map(|p| p.strip_prefix(&format!("{name}="))).unwrap().replace("%3A", ":").replace("%2F", "/");
    let address = param("redirect_uri").trim_start_matches("http://").to_string();
    let mut browser = TcpStream::connect(&address).unwrap();
    browser.write_all(format!("GET /?state={}&code=4%2F0AbCODE HTTP/1.1\r\n\r\n", param("state")).as_bytes()).unwrap();
    let mut rest = String::new();
    stderr.read_to_string(&mut rest).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{rest}");
    assert_eq!(String::from_utf8(out.stdout).unwrap(), format!("{REFRESH}\n"));
    assert!(rest.contains("vault"), "{rest}");
    for secret in [CLIENT_SECRET, REFRESH, "4/0AbCODE"] {
        assert!(!rest.contains(secret), "standard error carried {secret}");
    }
}

#[test]
fn an_unknown_flag_is_a_usage_error_before_anything_is_asked() {
    let _guard = support::guard("an_unknown_flag_is_a_usage_error_before_anything_is_asked");
    let home = tempfile::tempdir().unwrap();
    let google = loopback::serve(HashMap::new());
    let out = sign_in(&google.base, home.path()).arg("--open").stdin(Stdio::null()).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
    assert!(google.seen().is_empty());
}
```

Add `tempfile.workspace = true` under `[dev-dependencies]` in the package manifest.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-remote-gcal`
Expected: `environment variable CARGO_BIN_EXE_dam-gcal-sign-in not defined`, then, once the
`[[bin]]` entry and an empty `main` exist, the unit tests fail to compile on `client_id` and
`client_secret`.

- [ ] **Step 3: Implement**

```toml
# crates/dam-remote-gcal/Cargo.toml, after [lib]
[[bin]]
name = "dam-gcal-sign-in"
path = "src/bin/dam-gcal-sign-in.rs"
```

```rust
// crates/dam-remote-gcal/src/sign_in/command_line.rs
//! What the sign-in binary reads before it asks Google for anything: the
//! client id from its arguments, and the client secret from a pipe.

use std::io::Read;

use crate::Secret;

pub fn client_id(arguments: &[String]) -> Result<String, String> {
    let mut id = None;
    let mut words = arguments.iter();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--client-id" => id = Some(words.next().cloned().ok_or("--client-id takes the OAuth client id")?),
            w if w == "--client-secret" || w.starts_with("--client-secret=") => {
                return Err("the client secret is never an argument, where every process on this machine can read it; pipe it on standard input".into());
            }
            other => return Err(format!("{other} is not a flag this walk takes")),
        }
    }
    id.ok_or_else(|| "--client-id is required".to_string())
}

pub fn client_secret(mut input: impl Read, is_terminal: bool) -> Result<Secret, String> {
    if is_terminal {
        return Err("standard input is a terminal, where a typed secret echoes; pipe the client secret in".into());
    }
    let mut held = String::new();
    input.read_to_string(&mut held).map_err(|_| "standard input could not be read".to_string())?;
    let secret = held.trim();
    if secret.is_empty() {
        return Err("standard input carried no client secret".into());
    }
    Ok(Secret::from(secret))
}
```

```rust
// crates/dam-remote-gcal/src/bin/dam-gcal-sign-in.rs
//! The operator's one-time walk: consent in the browser, the refresh token on
//! standard output, and nothing written anywhere.

use std::io::{IsTerminal, Write};

use dam_remote_gcal::sign_in::{client_id, client_secret};
use dam_remote_gcal::{Client, Endpoints, SignIn};

const USAGE: &str = "usage: dam-gcal-sign-in --client-id <id>
  the client secret arrives on standard input, for example
  security find-generic-password -w -s 'Google dam client' | dam-gcal-sign-in --client-id <id>";

const KEEP_IT: &str = "Store the refresh token printed on standard output in your vault now: it is \
shown once and written nowhere. The gcal remote's refresh_token_command reads it from there.";

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let stdin = std::io::stdin();
    let is_terminal = stdin.is_terminal();
    let asked = client_id(&arguments).and_then(|id| Ok(Client { id, secret: client_secret(stdin.lock(), is_terminal)? }));
    let client = match asked {
        Ok(client) => client,
        Err(why) => {
            eprintln!("dam-gcal-sign-in: {why}\n{USAGE}");
            std::process::exit(2);
        }
    };
    let endpoints = match Endpoints::from_env() {
        Ok(e) => e,
        Err(why) => {
            eprintln!("dam-gcal-sign-in: {why}");
            std::process::exit(2);
        }
    };
    let minted = SignIn::new(endpoints).mint(&client, &mut |url| {
        eprintln!("Open this URL, grant access, and come back:\n\n{url}\n");
    });
    match minted {
        Ok(token) => {
            if writeln!(std::io::stdout().lock(), "{}", token.expose()).is_err() {
                std::process::exit(1);
            }
            eprintln!("{KEEP_IT}");
        }
        Err(why) => {
            eprintln!("dam-gcal-sign-in: {why}");
            std::process::exit(1);
        }
    }
}
```

README, a new section after "First run":

```markdown
## Google Calendar

    cargo install --git https://github.com/webdavis/damnit dam-remote-gcal

That installs `dam-remote-gcal`, which dam runs, and `dam-gcal-sign-in`, which you run once.

Google needs an OAuth client of type "Desktop app" with the Google Calendar API enabled on its
project. Keep its client id and client secret in your vault, then sign in:

    security find-generic-password -w -s "Google dam client" | dam-gcal-sign-in --client-id <id>

The command prints a URL; open it, grant read-only access to your calendar events, and the refresh
token is printed once on standard output. Store it in your vault. dam asks Google for read-only
access, so nothing it does can change your calendar.
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 5: Commit, and apply the amendment once approved**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal README.md
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): dam-gcal-sign-in prints a refresh token once and writes nothing"
```

When the operator has approved Amendment A4, apply its "Signing in" paragraph to the spec in a
separate commit, `docs(spec): say how a gcal refresh token is minted`. If A4 is still pending, stop
and ask; do not apply it, do not skip it.

---

### Task 4: dam hands each helper its remote's name and address

PR 2. Git invokes `git-remote-<transport> <remote-name> <address>`, where the address is the text
after `<transport>::` (gitremote-helpers(7), INVOCATION). dam does the same. A helper then reads its
credentials under its own remote's name, so a Google remote named `work` reads
`DAM_WORK_REFRESH_TOKEN`, and the address carries what a helper needs to know that is not a secret,
such as which calendars to read.

**Files:**
- Create: `crates/dam-protocol/src/invocation.rs`
- Modify: `crates/dam-protocol/src/lib.rs`, `crates/dam-protocol/docs/specs/protocol.md`
- Modify: `crates/dam-application/src/config.rs` (`RemoteConfig::address`)
- Modify: `crates/dam-adapters/src/helper_process.rs`, `crates/dam-adapters/src/lib.rs`
- Test: `crates/dam-protocol/src/invocation.rs`, `crates/dam-application/src/config.rs`,
  `crates/dam-adapters/src/helper_process.rs`

**Interfaces:**
- Produces:

```rust
// dam-protocol, invocation.rs: moved verbatim from dam-adapters, with its test
pub fn credential_variable(remote: &str, name: &str) -> String;   // DAM_<REMOTE>_<NAME>, uppercased, non-alphanumerics folded to _

// dam-application, config.rs
impl RemoteConfig {
    pub fn address(&self) -> &str;   // the text after `::` in the url, empty when there is none
}
```

- `dam-adapters` keeps `pub use dam_protocol::credential_variable;` in `lib.rs` so its callers are
  unchanged, and `ProcessLauncher::spawn` adds `command.arg(&remote.name.0).arg(remote.address())`.

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-application/src/config.rs, in the existing tests module
#[test]
fn the_address_is_the_text_after_the_helper_name() {
    let mut remote = RemoteConfig {
        name: RemoteName("gcal".into()),
        helper: "gcal".into(),
        url: "gcal::primary,team@group.calendar.google.com".into(),
        credentials: vec![],
        stale: None,
        deadline: None,
        path: None,
    };
    assert_eq!(remote.address(), "primary,team@group.calendar.google.com");
    remote.url = "todoist::".into();
    assert_eq!(remote.address(), "");
}
```

```rust
// crates/dam-adapters/src/helper_process.rs, in the existing tests module
/// Git's invocation: the remote's name, then the address its url names. A
/// helper reads its credentials under that name.
#[test]
fn the_helper_is_given_its_remote_name_and_address() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\nread -r line\nprintf '{\"protocol\":1,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[\"%s\",\"%s\",\"%s\"],\"incremental\":false}\\n' \"$#\" \"$1\" \"$2\"\n",
    );
    let mut remote = remote();
    remote.url = "t::primary,team@x".into();
    let mut helper = launcher.launch(&remote, &[]).unwrap();
    assert_eq!(
        helper.capabilities().unwrap().credentials,
        vec!["2".to_string(), remote.name.0.clone(), "primary,team@x".to_string()]
    );
}

#[test]
fn an_empty_address_is_still_passed_so_the_count_never_varies() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\nread -r line\nprintf '{\"protocol\":1,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[\"%s\",\"%s\"],\"incremental\":false}\\n' \"$#\" \"$2\"\n",
    );
    let mut helper = launcher.launch(&remote(), &[]).unwrap();
    assert_eq!(helper.capabilities().unwrap().credentials, vec!["2".to_string(), String::new()]);
}
```

`remote()` in `helper_process/testing.rs` builds a remote with url `t::`; the first test sets its own.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-application address && cargo test -p dam-adapters the_helper_is_given`
Expected: no method `address`; then the helper test fails with `["0", "", ""]`.

- [ ] **Step 3: Implement**

```rust
// crates/dam-application/src/config.rs
impl RemoteConfig {
    /// The text after `::` in the url, handed to the helper as its second
    /// argument the way git hands one to `git-remote-<transport>`.
    pub fn address(&self) -> &str {
        self.url.split_once("::").map_or("", |(_, address)| address)
    }
}
```

Move `credential_variable` and its `credential_variables_are_upper_snake` test from
`crates/dam-adapters/src/helper_process.rs` into `crates/dam-protocol/src/invocation.rs` unchanged,
with the doc comment "Where a helper reads one credential: `DAM_<REMOTE>_<NAME>`, uppercased with
every non-alphanumeric character folded to `_`." Export it from `dam-protocol`'s `lib.rs`; in
`dam-adapters` replace the definition with `use dam_protocol::credential_variable;` and keep
`pub use dam_protocol::credential_variable;` where `lib.rs` exported it. In `spawn`, after
`Command::new(program)`:

```rust
// Git's invocation: the remote's name, then the address its url names.
command.arg(&remote.name.0).arg(remote.address());
```

In `protocol.md`, a new section before "Version and compatibility":

```markdown
## Invocation

`dam` runs the helper as `dam-remote-<helper> <remote> <address>`: the remote's name as the first
argument and the text after `::` in its url as the second, empty when the url ends at `::`. Both are
always passed. A helper reads its credentials under that remote name, so one helper serves any number
of remotes. Neither argument is ever a secret.
```

The Todoist helper ignores its arguments today and keeps working unchanged.

- [ ] **Step 4: Run to verify pass**

Run: `just gates`
Expected: clean; every existing helper test still passes.

- [ ] **Step 5: Commit, and apply the amendment once approved**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-protocol crates/dam-application crates/dam-adapters
SKIP_AI_COMMIT=1 git commit -m "feat(helper): run each helper with its remote's name and address, as git does"
```

Once Amendment A1 is approved, apply it to the spec in its own commit,
`docs(spec): a helper is given its remote's name and address`. Pending: stop and ask.

---

### Task 5: An attendee records whether it is the calendar's own

PR 3. Google marks the attendee entry that "represents the calendar on which this copy of the event
appears" with `self`. Without it dam cannot tell a meeting the operator declined from one somebody
else declined, and a declined meeting would read as busy.

**Files:**
- Modify: `crates/dam-domain/src/object/event.rs` (`Attendee.is_self`)
- Modify: `crates/dam-protocol/src/wire.rs` (`WireAttendee.is_self`, `self` on the wire)
- Modify: `crates/dam-adapters/src/wire/encode.rs`, `crates/dam-adapters/src/wire/decode.rs`
- Modify: `crates/dam-application/src/merge.rs` (its test builds an `Attendee`)
- Test: `crates/dam-adapters/src/wire/tests.rs`, `crates/dam-protocol/src/messages/tests.rs`

**Interfaces:**
- Produces:

```rust
// dam-domain
pub struct Attendee { pub email: String, pub response: ResponseStatus, pub is_self: bool }

// dam-protocol
pub struct WireAttendee {
    pub email: String,
    pub response: String,
    #[serde(rename = "self", default, skip_serializing_if = "std::ops::Not::not")]
    pub is_self: bool,
}
```

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-protocol/src/messages/tests.rs
/// Written only when true, so every document without it reads exactly as
/// before, and a stored object from before the field reads as false.
#[test]
fn an_attendee_writes_self_only_when_it_is_the_calendars_own() {
    let own = crate::WireAttendee { email: "me@x".into(), response: "declined".into(), is_self: true };
    assert_eq!(serde_json::to_string(&own).unwrap(), r#"{"email":"me@x","response":"declined","self":true}"#);
    let other = crate::WireAttendee { is_self: false, ..own.clone() };
    assert_eq!(serde_json::to_string(&other).unwrap(), r#"{"email":"me@x","response":"declined"}"#);
    let read: crate::WireAttendee = serde_json::from_str(r#"{"email":"me@x","response":"accepted"}"#).unwrap();
    assert!(!read.is_self);
}
```

```rust
// crates/dam-adapters/src/wire/tests.rs
#[test]
fn the_calendars_own_attendee_round_trips() {
    let mut event = dam_domain::Event::new(oid(1), "standup", When::Day(date(2026, 9, 25)), When::Day(date(2026, 9, 26)));
    event.attendees = vec![
        Attendee { email: "me@x".into(), response: ResponseStatus::Declined, is_self: true },
        Attendee { email: "you@x".into(), response: ResponseStatus::Accepted, is_self: false },
    ];
    let object = Object::Event(event);
    assert_eq!(from_wire(&to_wire(&object, None)).unwrap(), object);
}
```

(`oid`, `date` and `When` come from what that test module already imports; add `Attendee` and
`ResponseStatus` to its `use dam_domain::{...}` line.)

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-protocol attendee && cargo test -p dam-adapters own_attendee`
Expected: no field `is_self` on either type.

- [ ] **Step 3: Implement**

Add the field to both structs as in the interface block, with the doc comment on the domain field:
"Whether this attendee is the calendar the event was read from: its answer is the calendar owner's."
In `encode.rs` map `is_self: a.is_self`; in `decode.rs` map `is_self: a.is_self`. Update the one
`Attendee { .. }` literal in `merge.rs`'s test with `is_self: false`. Compile errors name every other
site; each takes `is_self: false`.

- [ ] **Step 4: Run to verify pass**

Run: `just gates`
Expected: clean. The golden fixtures are unchanged, because a false `self` is never written.

- [ ] **Step 5: Commit, and apply the amendment once approved**

```bash
SKIP_AI_COMMIT=1 git add crates
SKIP_AI_COMMIT=1 git commit -m "feat(event): record which attendee is the calendar's own"
```

Once Amendment A2 is approved, apply it in its own commit, `docs(spec): an attendee says whether it
is the calendar's own`. Pending: stop and ask.

---

### Task 6: A pull can report an event cancelled by its id

PR 4. Google reports a deleted event as a cancelled item that is only guaranteed to carry its `id`.
dam's `removed` is a notice that stops tracking the object and leaves it as it was, so a deleted
meeting would stay confirmed and keep reading as busy. A pull gains `cancelled`: remote ids the
remote cancelled. dam builds the cancelled object from its own copy and lands it through the same
classification every pulled object takes, so an uncommitted local edit still stops the pull, a
committed one still raises a conflict, and attached tasks still raise the notice sync rule 4 names.

**Files:**
- Modify: `crates/dam-protocol/src/messages.rs` (`PullResponse.cancelled`, `PULL_KEYS`)
- Modify: `crates/dam-protocol/docs/specs/protocol.md`
- Create: `crates/dam-protocol/fixtures/pull-cancelled.response.json`
- Modify: `crates/dam-application/src/remote.rs` (`PullOutcome.cancelled`)
- Create: `crates/dam-application/src/use_cases/pull/cancelled.rs`
- Modify: `crates/dam-application/src/use_cases/pull.rs`
- Create: `crates/dam-application/src/use_cases/pull/tests/cancellations.rs`
- Modify: `crates/dam-adapters/src/wire/mod.rs` (`pull_from_wire`)
- Modify: `crates/dam-remote-todoist/src/pull.rs` (its `PullResponse` answers `cancelled: vec![]`)

**Interfaces:**
- Produces:

```rust
// dam-protocol
pub struct PullResponse {
    pub objects: Vec<WireObject>, pub removed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cancelled: Vec<String>,
    pub sync: Option<String>,
}

// dam-application, remote.rs
pub struct PullOutcome { /* existing fields */ pub cancelled: Vec<String> }

// dam-application, use_cases/pull/cancelled.rs
/// Each remote id the remote cancelled, as the event dam last knew under it
/// with its status set to cancelled. An id dam never mapped, and one mapped
/// to a task, yields nothing.
pub(super) fn cancelled_events(repos: Repositories<'_>, remote: &RemoteConfig, cancelled: &[String]) -> Result<Vec<IncomingObject>, UseCaseError>;
```

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-protocol/src/messages/tests.rs
#[test]
fn a_pull_carrying_only_cancellations_is_a_pull() {
    let read = serde_json::from_str::<Response>(r#"{"cancelled":["primary/e1"]}"#).unwrap();
    assert!(matches!(read, Response::Pull(p) if p.cancelled == vec!["primary/e1".to_string()]));
}

#[test]
fn pull_with_cancellations_round_trips_the_fixture() {
    let text = include_str!("../../fixtures/pull-cancelled.response.json").trim_end();
    let read: Response = serde_json::from_str(text).unwrap();
    assert_eq!(serde_json::to_string(&read).unwrap(), text);
}
```

Fixture, one line:

```json
{"objects":[],"removed":[],"cancelled":["primary/e1"],"sync":"tok-3"}
```

The existing `a_response_naming_no_known_shape_is_refused_rather_than_read_as_an_empty_pull` test
changes its expected text to `"objects, removed, cancelled or sync"`.

```rust
// crates/dam-application/src/use_cases/pull/tests/cancellations.rs
use std::cell::RefCell;
use std::rc::Rc;

use dam_domain::{EventStatus, Field, Kind, Object, Task, When};
use jiff::civil::date;

use super::*;

fn meeting() -> dam_domain::Event {
    dam_domain::Event::new(oid(5), "meeting", When::Day(date(2026, 9, 25)), When::Day(date(2026, 9, 26)))
}

/// The event as it was pulled and committed: working, committed and the
/// remote's snapshot all agree, and "e1" maps to it.
fn pulled(store: &MemoryStore, event: &dam_domain::Event) {
    store.put(&Object::Event(event.clone())).unwrap();
    add_all(store, store, store).unwrap();
    commit(store, store, &FixedClock(date(2026, 9, 18)), &FixedRandom::new(9), "pulled").unwrap();
    store.map_remote_id(&remote(), &event.base.oid, "e1").unwrap();
    store.set_remote_snapshot(&remote(), &Object::Event(event.clone())).unwrap();
}

fn cancelling(ids: &[&str]) -> ScriptedLauncher {
    let mut caps = task_caps();
    caps.kinds = vec![Kind::Event];
    caps.fields = vec![Field::Subject, Field::Start, Field::End, Field::Status];
    let answer = PullOutcome { cancelled: ids.iter().map(|s| s.to_string()).collect(), ..PullOutcome::default() };
    ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: caps.clone(),
            pull_answer: answer.clone(),
            push_answer: Box::new(|_| vec![]),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    }
}

fn run(store: &MemoryStore, ids: &[&str]) -> Result<Vec<PullReport>, UseCaseError> {
    pull(Repositories::of(store), &cancelling(ids), &EchoCredentials, &FixedClock(date(2026, 9, 18)), &FixedRandom::new(42), &config(), None)
}

#[test]
fn a_cancelled_id_moves_the_event_it_maps_to_cancelled() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let reports = run(&store, &["e1"]).unwrap();
    assert_eq!(reports[0].updated, 1);
    let Some(Object::Event(now)) = store.get(&oid(5)).unwrap() else { panic!("the event is gone") };
    assert_eq!(now.status, EventStatus::Cancelled);
    assert_eq!(now.base.subject, "meeting", "only the status moved");
    assert_eq!(store.oid_for_remote_id(&remote(), "e1").unwrap(), Some(oid(5)), "still tracked");
}

#[test]
fn a_cancelled_event_with_an_attached_task_is_reported_and_the_task_kept() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let mut prep = Task::new(oid(6), "prep");
    prep.event = Some(oid(5));
    store.put(&Object::Task(prep)).unwrap();
    run(&store, &["e1"]).unwrap();
    assert!(matches!(store.notices().unwrap().last(), Some(Notice::EventCancelled { attached: 1, .. })));
    assert!(store.get(&oid(6)).unwrap().is_some());
}

#[test]
fn an_id_dam_never_mapped_and_one_mapped_to_a_task_change_nothing() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(7), "a task"))).unwrap();
    store.map_remote_id(&remote(), &oid(7), "t1").unwrap();
    let reports = run(&store, &["never-seen", "t1"]).unwrap();
    assert_eq!((reports[0].created, reports[0].updated), (0, 0));
    assert!(matches!(store.get(&oid(7)).unwrap(), Some(Object::Task(_))));
}

#[test]
fn an_event_already_cancelled_is_unchanged() {
    let store = MemoryStore::new();
    let mut gone = meeting();
    gone.status = EventStatus::Cancelled;
    pulled(&store, &gone);
    assert_eq!(run(&store, &["e1"]).unwrap()[0].unchanged, 1);
}

#[test]
fn an_uncommitted_local_edit_stops_the_pull_as_any_upstream_change_would() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let mut edited = meeting();
    edited.base.subject = "renamed here".into();
    store.put(&Object::Event(edited)).unwrap();
    assert!(matches!(run(&store, &["e1"]).unwrap_err(), UseCaseError::Refused(Refusal::DirtyOnPull { .. })));
}
```

Register the file with `mod cancellations;` beside `mod kind_change;` in `pull/tests.rs`.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-protocol cancell && cargo test -p dam-application cancellations`
Expected: no field `cancelled` on `PullResponse` or `PullOutcome`.

- [ ] **Step 3: Implement**

In `messages.rs`, add the field as in the interface, extend `PULL_KEYS` to
`["objects", "removed", "cancelled", "sync"]` (its type becomes `[&str; 4]`), and change the refusal
text to `"a response naming none of objects, removed, cancelled or sync, over {} keys"`. Add
`pub cancelled: Vec<String>` to `PullOutcome` after `removed`, documented "Remote ids the remote
cancelled: each moves the event it maps to cancelled, through the same rules as any pulled change."
In `pull_from_wire`, `cancelled: response.cancelled`. Every struct literal of `PullResponse` or
`PullOutcome` that does not end in `..Default::default()` gains `cancelled: vec![]`: two in
`dam-protocol`'s message tests, four in `dam-application`'s pull tests, and the Todoist helper's
`crates/dam-remote-todoist/src/pull.rs`, which answers no cancellations.

```rust
// crates/dam-application/src/use_cases/pull/cancelled.rs
//! A cancellation the remote reported by id alone, as the whole object the
//! rest of the pull already knows how to land.

use dam_domain::{EventStatus, Object};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Repositories;
use crate::remote::IncomingObject;

/// The base is the remote's own last snapshot, which is what the remote
/// held before it cancelled; the working copy stands in when no snapshot was
/// kept. Either way only the status differs, so classification sees exactly
/// one upstream change.
pub(super) fn cancelled_events(repos: Repositories<'_>, remote: &RemoteConfig, cancelled: &[String]) -> Result<Vec<IncomingObject>, UseCaseError> {
    let mut out = Vec::new();
    for remote_id in cancelled {
        let Some(oid) = repos.remote_tracking.oid_for_remote_id(&remote.name, remote_id)? else {
            continue;
        };
        let known = match repos.remote_tracking.remote_snapshot(&remote.name, &oid)? {
            Some(snapshot) => Some(snapshot),
            None => repos.objects.get(&oid)?,
        };
        if let Some(Object::Event(mut event)) = known {
            event.status = EventStatus::Cancelled;
            out.push(IncomingObject { remote_id: remote_id.clone(), object: Object::Event(event) });
        }
    }
    Ok(out)
}
```

In `pull.rs`, add `mod cancelled;` and in `land_everything`, replace
`let objects = std::mem::take(&mut response.objects);` with:

```rust
let mut objects = std::mem::take(&mut response.objects);
objects.extend(cancelled::cancelled_events(repos, remote, &response.cancelled)?);
```

In `protocol.md`, the pull row of the responses table becomes `objects`, `removed`, `cancelled` or
`sync` / `objects`, `removed`, `cancelled`, `sync`, and a paragraph follows the table:

```markdown
`cancelled` lists remote ids the remote cancelled, for a remote that reports a cancellation without
restating the object, as Google Calendar does for a deleted event. `dam` moves the event it tracks
under each id to `cancelled` and keeps tracking it; an id it does not track, or one that names a
task, changes nothing. `removed` stays what it was: the remote no longer has the object, and `dam`
stops tracking it.
```

- [ ] **Step 4: Run to verify pass**

Run: `just gates`
Expected: clean.

- [ ] **Step 5: Commit, and apply the amendment once approved**

```bash
SKIP_AI_COMMIT=1 git add crates
SKIP_AI_COMMIT=1 git commit -m "feat(pull): a helper can report an event cancelled by its remote id"
```

Once Amendment A3 is approved, apply it in its own commit, `docs(spec): a pull reports a
cancellation by id`. Pending: stop and ask.

---

### Task 7: What the helper is given

PR 5. Tasks 7 to 12 need PRs 1 to 4 merged: this branch starts from a `main` that has them.

**Files:**
- Create: `crates/dam-remote-gcal/src/address.rs`, `src/credentials.rs`, `src/capabilities.rs`
- Modify: `crates/dam-remote-gcal/src/lib.rs`

**Interfaces:**
- Consumes: `dam_protocol::{credential_variable, Capabilities, PROTOCOL_VERSION}`, `Secret`, `Client`.
- Produces:

```rust
// address.rs
pub fn calendars(address: &str) -> Vec<String>;   // comma-separated ids, trimmed, first occurrence kept; none means ["primary"]

// credentials.rs
pub struct Credentials { pub client: Client, pub refresh_token: Secret }
impl Credentials {
    pub fn from_env(remote: &str) -> Result<Credentials, String>;   // from_lookup over std::env::var
    pub fn from_lookup(remote: &str, lookup: impl Fn(&str) -> Option<String>) -> Result<Credentials, String>;   // every variable named, no value ever
}

// capabilities.rs
pub fn capabilities() -> Capabilities;   // kinds ["event"], the sixteen event fields, the three credentials, incremental true
```

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-remote-gcal/src/address.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_address_reads_the_primary_calendar() {
        assert_eq!(calendars(""), vec!["primary"]);
        assert_eq!(calendars(" , "), vec!["primary"]);
    }

    #[test]
    fn named_calendars_are_read_in_order_once_each() {
        assert_eq!(
            calendars("primary, team@group.calendar.google.com,primary,en.usa#holiday@group.v.calendar.google.com"),
            vec!["primary", "team@group.calendar.google.com", "en.usa#holiday@group.v.calendar.google.com"]
        );
    }
}
```

```rust
// crates/dam-remote-gcal/src/credentials.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    fn environment(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let held: HashMap<String, String> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |name| held.get(name).cloned()
    }

    #[test]
    fn credentials_are_read_under_the_remotes_own_name() {
        let lookup = environment(&[
            ("DAM_WORK_CAL_CLIENT_ID", "id-1"),
            ("DAM_WORK_CAL_CLIENT_SECRET", "GOCSPX-S"),
            ("DAM_WORK_CAL_REFRESH_TOKEN", "1//0gR"),
        ]);
        let c = Credentials::from_lookup("work-cal", lookup).unwrap();
        assert_eq!(c.client.id, "id-1");
        assert_eq!(c.client.secret.expose(), "GOCSPX-S");
        assert_eq!(c.refresh_token.expose(), "1//0gR");
    }

    #[test]
    fn a_missing_credential_names_its_variable_and_its_config_key() {
        let lookup = environment(&[("DAM_HALF_CLIENT_ID", "id-1"), ("DAM_HALF_CLIENT_SECRET", "GOCSPX-S"), ("DAM_HALF_REFRESH_TOKEN", "")]);
        let said = Credentials::from_lookup("half", lookup).unwrap_err();
        assert!(said.contains("DAM_HALF_REFRESH_TOKEN"), "{said}");
        assert!(said.contains("refresh_token under [remote.half]"), "{said}");
        assert!(!said.contains("GOCSPX-S"), "{said}");
    }
}
```

```rust
// crates/dam-remote-gcal/src/capabilities.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_only_with_every_google_field_and_the_three_credentials() {
        let caps = capabilities();
        assert!(caps.supported());
        assert_eq!(caps.kinds, vec!["event"]);
        assert_eq!(caps.credentials, vec!["client_id", "client_secret", "refresh_token"]);
        assert!(caps.incremental);
        for field in ["subject", "body", "path", "start", "end", "timezone", "location", "attendees", "status", "transparency", "visibility", "event_type", "color", "organizer", "conference", "attachments"] {
            assert!(caps.fields.iter().any(|f| f == field), "{field}");
        }
        for dam_only in ["labels", "depends", "reminders", "recurrence"] {
            assert!(!caps.fields.iter().any(|f| f == dam_only), "{dam_only} is dam's own");
        }
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-remote-gcal`
Expected: unresolved `calendars`, `Credentials`, `capabilities`.

- [ ] **Step 3: Implement**

```rust
// crates/dam-remote-gcal/src/address.rs
//! The remote's address names the calendars it reads: `gcal::` is the
//! primary calendar, `gcal::primary,team@group.calendar.google.com` two.

pub fn calendars(address: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in address.split(',').map(str::trim).filter(|id| !id.is_empty()) {
        if !out.iter().any(|seen| seen == id) {
            out.push(id.to_string());
        }
    }
    if out.is_empty() {
        out.push("primary".into());
    }
    out
}
```

```rust
// crates/dam-remote-gcal/src/credentials.rs
//! The three credentials dam resolved and handed over in the environment,
//! under this remote's own name.

use dam_protocol::credential_variable;

use crate::{Client, Secret};

#[derive(Debug)]
pub struct Credentials {
    pub client: Client,
    pub refresh_token: Secret,
}

impl Credentials {
    pub fn from_env(remote: &str) -> Result<Credentials, String> {
        Credentials::from_lookup(remote, |variable| std::env::var(variable).ok())
    }

    /// An empty value is no value: dam never hands one over, so it is a
    /// variable set by something else.
    pub fn from_lookup(remote: &str, lookup: impl Fn(&str) -> Option<String>) -> Result<Credentials, String> {
        let read = |name: &str| {
            let variable = credential_variable(remote, name);
            lookup(&variable)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| format!("{variable} is not set; declare {name} under [remote.{remote}]"))
        };
        Ok(Credentials {
            client: Client { id: read("client_id")?, secret: Secret::from(read("client_secret")?) },
            refresh_token: Secret::from(read("refresh_token")?),
        })
    }
}
```

```rust
// crates/dam-remote-gcal/src/capabilities.rs
use dam_protocol::{Capabilities, PROTOCOL_VERSION};

/// Events, every field Google's event resource carries that dam models, and
/// none of dam's own fields, so labels, dependencies, reminders and
/// recurrence stay in dam. Incremental: the helper reads `since` and answers
/// a token.
pub fn capabilities() -> Capabilities {
    let fields = [
        "subject", "body", "path", "start", "end", "timezone", "location", "attendees",
        "status", "transparency", "visibility", "event_type", "color", "organizer",
        "conference", "attachments",
    ];
    Capabilities {
        protocol: PROTOCOL_VERSION,
        kinds: vec!["event".into()],
        fields: fields.iter().map(|f| f.to_string()).collect(),
        credentials: vec!["client_id".into(), "client_secret".into(), "refresh_token".into()],
        incremental: true,
    }
}
```

`lib.rs` gains `pub mod address; pub mod capabilities; pub mod credentials;` and
`pub use credentials::Credentials;`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 5: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): read the calendars from the address and the credentials under the remote's name"
```

---

### Task 8: The access token

PR 5. One `refresh_token` grant per pull. A pull is one helper process, so there is no cache to keep.

**Files:**
- Create: `crates/dam-remote-gcal/src/access.rs`, `src/api_error.rs`
- Create: `crates/dam-remote-gcal/tests/api.rs`
- Modify: `crates/dam-remote-gcal/src/lib.rs`

**Interfaces:**
- Consumes: `http::{agent, read, MAX_ANSWER}`, `encoding::form`, `oauth_error::oauth_error_code`,
  `Credentials`, `Endpoints`.
- Produces:

```rust
// api_error.rs
#[derive(Debug)]
pub enum ApiError {
    TokenRefused { status: Option<u16>, code: Option<&'static str> },
    Http { status: u16, body: String },                 // the body cut to one line of at most 500 characters
    RateLimited { retry_after: Option<std::time::Duration>, body: String },
    Transport(String),
    Decode(String),
    TooLarge { limit: u64 },
    TooManyPages { limit: usize },
}
impl std::fmt::Display for ApiError;   // TokenRefused never quotes the body; invalid_grant says to sign in again

// access.rs
pub fn access_token(agent: &ureq::Agent, endpoints: &Endpoints, credentials: &Credentials) -> Result<Secret, ApiError>;
```

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-remote-gcal/tests/api.rs
mod loopback;
mod support;

use std::collections::HashMap;

use dam_remote_gcal::access::access_token;
use dam_remote_gcal::{ApiError, Client, Credentials, Endpoints, http_agent};
use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn credentials() -> Credentials {
    Credentials {
        client: Client { id: "123.apps".into(), secret: CLIENT_SECRET.into() },
        refresh_token: REFRESH.into(),
    }
}

fn token_route(reply: Reply) -> loopback::Loopback {
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![reply]);
    loopback::serve(routes)
}

#[test]
fn the_refresh_grant_sends_the_three_credentials_in_a_form_and_reads_the_access_token() {
    let _guard = support::guard("the_refresh_grant_sends_the_three_credentials_in_a_form_and_reads_the_access_token");
    let google = token_route(Reply::json(200, serde_json::json!({"access_token": "ya29.ACCESS", "expires_in": 3599, "token_type": "Bearer"})));
    let token = access_token(&http_agent(), &Endpoints::loopback(&google.base).unwrap(), &credentials()).unwrap();
    assert_eq!(token.expose(), "ya29.ACCESS");
    let body = &google.seen()[0].body;
    for field in ["grant_type=refresh_token", "client_id=123.apps", "client_secret=GOCSPX-SUPERSECRETCLIENT", "refresh_token=1%2F%2F0gSUPERSECRETREFRESH"] {
        assert!(body.contains(field), "{field}");
    }
}

#[test]
fn a_revoked_refresh_token_says_to_sign_in_again_and_quotes_nothing() {
    let _guard = support::guard("a_revoked_refresh_token_says_to_sign_in_again_and_quotes_nothing");
    let google = token_route(Reply::json(400, serde_json::json!({
        "error": "invalid_grant", "error_description": format!("Token has been expired or revoked. {REFRESH} {CLIENT_SECRET}")
    })));
    let err = access_token(&http_agent(), &Endpoints::loopback(&google.base).unwrap(), &credentials()).unwrap_err();
    let said = err.to_string();
    assert!(said.contains("invalid_grant") && said.contains("dam-gcal-sign-in"), "{said}");
    for secret in [REFRESH, CLIENT_SECRET, "expired or revoked"] {
        assert!(!said.contains(secret), "{said}");
    }
}

#[test]
fn an_answer_without_an_access_token_is_refused_without_quoting_it() {
    let _guard = support::guard("an_answer_without_an_access_token_is_refused_without_quoting_it");
    let google = token_route(Reply::json(200, serde_json::json!({"token_type": "Bearer", "echo": REFRESH})));
    let err = access_token(&http_agent(), &Endpoints::loopback(&google.base).unwrap(), &credentials()).unwrap_err();
    assert!(matches!(err, ApiError::TokenRefused { status: Some(200), code: None }), "{err:?}");
    assert!(!err.to_string().contains(REFRESH));
}

#[test]
fn an_error_never_debug_prints_a_credential() {
    let _guard = support::guard("an_error_never_debug_prints_a_credential");
    let google = token_route(Reply::json(401, serde_json::json!({"error": "invalid_client", "e": CLIENT_SECRET})));
    let err = access_token(&http_agent(), &Endpoints::loopback(&google.base).unwrap(), &credentials()).unwrap_err();
    assert!(!format!("{err:?}").contains(CLIENT_SECRET));
}
```

`http_agent` is the crate's one exported constructor for the agent every call rides, so a test
builds the same agent production does: `pub use http::agent as http_agent;` in `lib.rs`.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-remote-gcal --test api`
Expected: unresolved `access`, `ApiError`, `http_agent`.

- [ ] **Step 3: Implement**

```rust
// crates/dam-remote-gcal/src/access.rs
//! The refresh_token grant: the three credentials in a form body, an access
//! token out. The body of a refused answer is where a token endpoint echoes
//! credentials back, so it is never quoted.

use crate::api_error::ApiError;
use crate::encoding::form;
use crate::http::{MAX_ANSWER, read};
use crate::oauth_error::oauth_error_code;
use crate::{Credentials, Endpoints, Secret};

pub fn access_token(agent: &ureq::Agent, endpoints: &Endpoints, credentials: &Credentials) -> Result<Secret, ApiError> {
    let body = form(&[
        ("grant_type", "refresh_token"),
        ("client_id", &credentials.client.id),
        ("client_secret", credentials.client.secret.expose()),
        ("refresh_token", credentials.refresh_token.expose()),
    ]);
    let refused = |status| ApiError::TokenRefused { status, code: None };
    let response = agent
        .post(&endpoints.token)
        .header("content-type", "application/x-www-form-urlencoded")
        .send(body.as_str())
        .map_err(|_| refused(None))?;
    let answer = read(response, MAX_ANSWER).map_err(|_| refused(None))?;
    if !(200..300).contains(&answer.status) {
        return Err(ApiError::TokenRefused { status: Some(answer.status), code: oauth_error_code(&answer.body) });
    }
    serde_json::from_str::<serde_json::Value>(&answer.body)
        .ok()
        .and_then(|v| v.get("access_token")?.as_str().filter(|t| !t.is_empty()).map(Secret::from))
        .ok_or(refused(Some(answer.status)))
}
```

`api_error.rs` holds the enum from the interface block and this `Display`:

```rust
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::TokenRefused { code: Some("invalid_grant"), status } => write!(
                f,
                "Google refused the refresh token (HTTP {}, invalid_grant): it was revoked or has expired; run dam-gcal-sign-in again and store the new one",
                status.map_or_else(|| "?".to_string(), |s| s.to_string())
            ),
            ApiError::TokenRefused { status: Some(s), code: Some(c) } => write!(f, "Google refused the token exchange (HTTP {s}, {c})"),
            ApiError::TokenRefused { status: Some(s), code: None } => write!(f, "Google refused the token exchange (HTTP {s})"),
            ApiError::TokenRefused { status: None, .. } => f.write_str("the token exchange did not reach Google or its answer could not be read"),
            ApiError::Http { status, body } => write!(f, "Google Calendar answered {status}: {body}"),
            ApiError::RateLimited { retry_after: Some(d), .. } => write!(f, "Google Calendar is rate limiting this account; retry after {}s", d.as_secs()),
            ApiError::RateLimited { retry_after: None, body } => write!(f, "Google Calendar is rate limiting this account and named no retry time: {body}"),
            ApiError::Transport(s) => write!(f, "reaching Google Calendar: {s}"),
            ApiError::Decode(s) => write!(f, "reading Google Calendar's answer: {s}"),
            ApiError::TooLarge { limit } => write!(f, "Google Calendar's answer is past the {limit} byte ceiling one pull can carry"),
            ApiError::TooManyPages { limit } => write!(f, "Google Calendar kept paging past {limit} pages, so the pull stopped"),
        }
    }
}
```

`lib.rs` gains `pub mod access; mod api_error; pub use api_error::ApiError; pub use http::agent as
http_agent;`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 5: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): exchange the refresh token for an access token, quoting nothing back"
```

---

### Task 9: Listing a calendar's events

PR 5.

**Files:**
- Create: `crates/dam-remote-gcal/src/calendar_api.rs`, `src/calendar_api/resources.rs`
- Modify: `crates/dam-remote-gcal/Cargo.toml` (`jiff.workspace = true`), `src/lib.rs`
- Modify: `crates/dam-remote-gcal/tests/api.rs`

**Interfaces:**
- Consumes: `Secret`, `Endpoints`, `ApiError`, `http::{read, MAX_ANSWER}`, `encoding::{form, percent_encoded}`.
- Produces:

```rust
// calendar_api.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window { pub from: jiff::Timestamp, pub to: jiff::Timestamp }
#[derive(Debug)]
pub struct Listing { pub calendar: String, pub summary: Option<String>, pub time_zone: Option<String>, pub items: Vec<GoogleEvent> }
pub const PAGE_SIZE: u32 = 2500;
pub const MAX_PAGES: usize = 64;
pub struct CalendarApi { /* agent, base, access */ }
impl CalendarApi {
    pub fn new(agent: ureq::Agent, endpoints: &Endpoints, access: Secret) -> CalendarApi;
    pub fn events(&self, calendar: &str, window: &Window) -> Result<Listing, ApiError>;
}

// resources.rs: Google's event resource, camelCase on the wire, every field optional but id
pub struct EventsPage { pub summary: Option<String>, pub time_zone: Option<String>, pub items: Vec<GoogleEvent>, pub next_page_token: Option<String> }
pub struct GoogleEvent { pub id: String, pub status: Option<String>, pub summary: Option<String>, pub description: Option<String>, pub location: Option<String>, pub start: Option<GoogleTime>, pub end: Option<GoogleTime>, pub transparency: Option<String>, pub visibility: Option<String>, pub event_type: Option<String>, pub color_id: Option<String>, pub organizer: Option<GooglePerson>, pub attendees: Vec<GoogleAttendee>, pub conference_data: Option<ConferenceData>, pub attachments: Vec<GoogleAttachment> }
pub struct GoogleTime { pub date: Option<String>, pub date_time: Option<String>, pub time_zone: Option<String> }
pub struct GooglePerson { pub email: Option<String>, pub display_name: Option<String> }
pub struct GoogleAttendee { pub email: Option<String>, pub response_status: Option<String>, pub is_self: bool }   // `self` on the wire
pub struct ConferenceData { pub conference_solution: Option<ConferenceSolution>, pub entry_points: Vec<EntryPoint> }
pub struct ConferenceSolution { pub name: Option<String> }
pub struct EntryPoint { pub entry_point_type: Option<String>, pub uri: Option<String> }
pub struct GoogleAttachment { pub file_url: String, pub title: Option<String>, pub mime_type: Option<String> }
```

- [ ] **Step 1: Write the failing tests**

Append to `tests/api.rs`:

```rust
use dam_remote_gcal::calendar_api::{CalendarApi, MAX_PAGES, Window};

fn window() -> Window {
    Window { from: "2026-09-15T00:00:00Z".parse().unwrap(), to: "2026-12-14T00:00:00Z".parse().unwrap() }
}

fn api(base: &str) -> CalendarApi {
    CalendarApi::new(http_agent(), &Endpoints::loopback(base).unwrap(), "ya29.ACCESS".into())
}

fn page(items: serde_json::Value, next: Option<&str>) -> Reply {
    let mut body = serde_json::json!({"summary": "me@example.com", "timeZone": "America/New_York", "items": items});
    if let Some(n) = next {
        body["nextPageToken"] = serde_json::json!(n);
    }
    Reply::json(200, body)
}

#[test]
fn one_calendar_is_asked_for_single_instances_deleted_included_over_the_window() {
    let _guard = support::guard("one_calendar_is_asked_for_single_instances_deleted_included_over_the_window");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", vec![page(serde_json::json!([{"id": "e1"}]), None)]);
    let google = loopback::serve(routes);
    let listing = api(&google.base).events("primary", &window()).unwrap();
    assert_eq!(listing.summary.as_deref(), Some("me@example.com"));
    assert_eq!(listing.time_zone.as_deref(), Some("America/New_York"));
    assert_eq!(listing.items[0].id, "e1");
    let seen = &google.seen()[0];
    assert_eq!(seen.authorization.as_deref(), Some("Bearer ya29.ACCESS"));
    for pair in ["singleEvents=true", "showDeleted=true", "maxResults=2500", "timeMin=2026-09-15T00%3A00%3A00Z", "timeMax=2026-12-14T00%3A00%3A00Z"] {
        assert!(seen.query.contains(pair), "{pair} missing from {}", seen.query);
    }
}

#[test]
fn every_page_is_read_until_google_names_no_next_one() {
    let _guard = support::guard("every_page_is_read_until_google_names_no_next_one");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", vec![
        page(serde_json::json!([{"id": "e1"}]), Some("p2")),
        page(serde_json::json!([{"id": "e2"}]), None),
    ]);
    let google = loopback::serve(routes);
    let listing = api(&google.base).events("primary", &window()).unwrap();
    assert_eq!(listing.items.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), vec!["e1", "e2"]);
    assert!(google.seen()[1].query.contains("pageToken=p2"));
}

#[test]
fn a_calendar_that_pages_forever_is_stopped_at_the_cap() {
    let _guard = support::guard("a_calendar_that_pages_forever_is_stopped_at_the_cap");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", vec![page(serde_json::json!([]), Some("again"))]);
    let google = loopback::serve(routes);
    let err = api(&google.base).events("primary", &window()).unwrap_err();
    assert!(matches!(err, ApiError::TooManyPages { limit } if limit == MAX_PAGES), "{err:?}");
    assert_eq!(google.seen().len(), MAX_PAGES);
}

#[test]
fn a_calendar_id_is_one_percent_encoded_path_segment() {
    let _guard = support::guard("a_calendar_id_is_one_percent_encoded_path_segment");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/en.usa%23holiday%40group.v.calendar.google.com/events", vec![page(serde_json::json!([]), None)]);
    let google = loopback::serve(routes);
    let listing = api(&google.base).events("en.usa#holiday@group.v.calendar.google.com", &window()).unwrap();
    assert_eq!(listing.calendar, "en.usa#holiday@group.v.calendar.google.com");
}

#[test]
fn a_calendar_google_does_not_know_names_itself_in_the_error() {
    let _guard = support::guard("a_calendar_google_does_not_know_names_itself_in_the_error");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/nope/events", vec![Reply::json(404, serde_json::json!({"error": {"code": 404, "message": "Not Found"}}))]);
    let google = loopback::serve(routes);
    let said = api(&google.base).events("nope", &window()).unwrap_err().to_string();
    assert!(said.contains("404") && said.contains("Not Found"), "{said}");
}

#[test]
fn a_rate_limit_carries_the_retry_time_google_named() {
    let _guard = support::guard("a_rate_limit_carries_the_retry_time_google_named");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", vec![Reply::json(429, serde_json::json!({})).with_header("Retry-After", "30")]);
    let google = loopback::serve(routes);
    let err = api(&google.base).events("primary", &window()).unwrap_err();
    assert!(err.to_string().contains("retry after 30s"), "{err}");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-remote-gcal --test api`
Expected: unresolved `calendar_api`.

- [ ] **Step 3: Implement**

`resources.rs` declares the structs in the interface block with `#[derive(Clone, Debug, Default,
Deserialize)]` and `#[serde(rename_all = "camelCase")]`, `#[serde(default)]` on every field but
`GoogleEvent::id` and `GoogleAttachment::file_url`, and `#[serde(rename = "self")]` on
`GoogleAttendee::is_self`. Unknown fields are ignored, as serde does by default.

```rust
// crates/dam-remote-gcal/src/calendar_api.rs
//! Calendar API v3 `events.list` over one calendar and one window.

mod resources;

pub use resources::{ConferenceData, ConferenceSolution, EntryPoint, EventsPage, GoogleAttachment, GoogleAttendee, GoogleEvent, GooglePerson, GoogleTime};

use crate::api_error::{ApiError, bounded};
use crate::encoding::{form, percent_encoded};
use crate::http::{MAX_ANSWER, ReadError, read};
use crate::{Endpoints, Secret};

/// The most `events.list` answers in one page.
pub const PAGE_SIZE: u32 = 2500;
/// How many pages one calendar may take. At the page size above that is
/// 160,000 events, far past a pull's line, and it stops a server that names a
/// next page forever.
pub const MAX_PAGES: usize = 64;

const TOO_MANY_REQUESTS: u16 = 429;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: jiff::Timestamp,
    pub to: jiff::Timestamp,
}

#[derive(Debug)]
pub struct Listing {
    pub calendar: String,
    pub summary: Option<String>,
    pub time_zone: Option<String>,
    pub items: Vec<GoogleEvent>,
}

pub struct CalendarApi {
    agent: ureq::Agent,
    base: String,
    access: Secret,
}

impl CalendarApi {
    pub fn new(agent: ureq::Agent, endpoints: &Endpoints, access: Secret) -> CalendarApi {
        CalendarApi { agent, base: endpoints.calendar.clone(), access }
    }

    /// Every event whose end is after `window.from` and whose start is
    /// before `window.to`, recurring events expanded into instances, cancelled
    /// ones included, over as many pages as Google names.
    pub fn events(&self, calendar: &str, window: &Window) -> Result<Listing, ApiError> {
        let url = format!("{}/calendars/{}/events", self.base, percent_encoded(calendar));
        let (from, to) = (window.from.to_string(), window.to.to_string());
        let size = PAGE_SIZE.to_string();
        let mut listing = Listing { calendar: calendar.to_string(), summary: None, time_zone: None, items: Vec::new() };
        let mut next: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let mut query = vec![("singleEvents", "true"), ("showDeleted", "true"), ("maxResults", size.as_str()), ("timeMin", from.as_str()), ("timeMax", to.as_str())];
            if let Some(token) = &next {
                query.push(("pageToken", token.as_str()));
            }
            let page = self.page(&format!("{url}?{}", form(&query)))?;
            listing.summary = listing.summary.or(page.summary);
            listing.time_zone = listing.time_zone.or(page.time_zone);
            listing.items.extend(page.items);
            match page.next_page_token {
                Some(token) => next = Some(token),
                None => return Ok(listing),
            }
        }
        Err(ApiError::TooManyPages { limit: MAX_PAGES })
    }

    fn page(&self, url: &str) -> Result<EventsPage, ApiError> {
        let response = self
            .agent
            .get(url)
            .header("authorization", &format!("Bearer {}", self.access.expose()))
            .call()
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        let answer = read(response, MAX_ANSWER).map_err(|e| match e {
            ReadError::TooLarge { limit } => ApiError::TooLarge { limit },
            ReadError::Transport(s) => ApiError::Transport(s),
        })?;
        if answer.status == TOO_MANY_REQUESTS {
            return Err(ApiError::RateLimited { retry_after: answer.retry_after, body: bounded(&answer.body) });
        }
        if !(200..300).contains(&answer.status) {
            return Err(ApiError::Http { status: answer.status, body: bounded(&answer.body) });
        }
        serde_json::from_str(&answer.body).map_err(|e| ApiError::Decode(e.to_string()))
    }
}
```

`jiff::Timestamp`'s `Display` writes RFC 3339 in UTC with a `Z`, which is what `timeMin` and
`timeMax` take. `lib.rs` gains `pub mod calendar_api;`. `api_error.rs` gains `pub(crate) fn
bounded`, copied from `crates/dam-remote-todoist/src/api/error.rs` with its `MAX_ERROR_BODY` of 500
(one line, control characters flattened, a cut marked with an ellipsis). `http.rs` gains the
`Retry-After` reading, the only change to Task 2's code:

```rust
// crates/dam-remote-gcal/src/http.rs
pub(crate) struct Answer {
    pub(crate) status: u16,
    pub(crate) retry_after: Option<Duration>,
    pub(crate) body: String,
}

// in read(), before the body is consumed:
let retry_after = response
    .headers()
    .get("retry-after")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.trim().parse().ok())
    .map(Duration::from_secs);
// and the answer:
Ok(Answer { status, retry_after, body })
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 5: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal Cargo.lock
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): list a calendar's events over a window, page by bounded page"
```

---

### Task 10: Mapping events, and the pull

PR 5. A remote id is `<calendar>/<event id>`: event ids are unique per calendar, not across
calendars, and Google's event ids never contain `/`, so the id splits at its last `/`. An event's
path is its calendar's title as one segment, the way the Todoist helper names a project.

**Files:**
- Create: `crates/dam-remote-gcal/src/map.rs`, `src/map/time.rs`, `src/pull.rs`
- Create: `crates/dam-remote-gcal/tests/pull.rs`
- Modify: `crates/dam-remote-gcal/src/lib.rs`

**Interfaces:**
- Consumes: `Listing`, `GoogleEvent`, `CalendarApi`, `Window`, `ApiError`,
  `dam_protocol::{WireObject, WireEvent, WireAttendee, WirePerson, WireConference, WireAttachment, PullResponse}`.
- Produces:

```rust
// map.rs
#[derive(Debug)]
pub(crate) enum Mapped { Live { object: Box<WireObject>, end: i64 }, Cancelled(String) }   // boxed: the variants differ by 700 bytes
#[derive(Debug, PartialEq, Eq)]
pub struct MapError { pub calendar: String, pub event_id: String, pub what: &'static str }   // Display: "calendar <c>, event <id>: <what>"
pub fn remote_id(calendar: &str, event_id: &str) -> String;
pub(crate) fn calendar_of(remote_id: &str) -> Option<&str>;
pub(crate) fn map_event(listing: &Listing, event: &GoogleEvent) -> Result<Mapped, MapError>;

// map/time.rs
pub(super) fn when(time: &GoogleTime, calendar_zone: Option<&str>) -> Result<(String, i64), &'static str>;   // text dam reads, epoch seconds

// pull.rs
pub const LOOKBACK_DAYS: i64 = 7;
pub const HORIZON_DAYS: i64 = 90;
pub fn window(now: jiff::Timestamp) -> Window;
#[derive(Debug)]
pub enum PullError { Api(ApiError), Map(MapError) }   // Display delegates
pub fn pull(api: &CalendarApi, calendars: &[String], since: Option<&str>, now: jiff::Timestamp) -> Result<PullResponse, PullError>;
```

- [ ] **Step 1: Write the failing mapping tests**

```rust
// crates/dam-remote-gcal/src/map.rs, at the bottom
#[cfg(test)]
mod tests;
```

```rust
// crates/dam-remote-gcal/src/map/tests.rs
use super::*;
use crate::calendar_api::{GoogleEvent, Listing};

fn listing() -> Listing {
    Listing { calendar: "primary".into(), summary: Some("me@example.com".into()), time_zone: Some("America/New_York".into()), items: vec![] }
}

fn google(value: serde_json::Value) -> GoogleEvent {
    serde_json::from_value(value).unwrap()
}

fn live(value: serde_json::Value) -> (WireObject, i64) {
    match map_event(&listing(), &google(value)).unwrap() {
        Mapped::Live { object, end } => (*object, end),
        Mapped::Cancelled(id) => panic!("cancelled {id}"),
    }
}

#[test]
fn a_timed_event_maps_every_field_dam_models() {
    let (w, end) = live(serde_json::json!({
        "id": "e1", "status": "tentative", "summary": "Standup", "description": "notes", "location": "Room 4",
        "start": {"dateTime": "2026-09-25T14:00:00-04:00", "timeZone": "America/New_York"},
        "end": {"dateTime": "2026-09-25T14:30:00-04:00", "timeZone": "America/New_York"},
        "transparency": "opaque", "visibility": "private", "eventType": "focusTime", "colorId": "5",
        "organizer": {"email": "boss@example.com", "displayName": "Boss"},
        "attendees": [
            {"email": "me@example.com", "responseStatus": "declined", "self": true},
            {"email": "you@example.com", "responseStatus": "needsAction"},
            {"responseStatus": "accepted"}
        ],
        "conferenceData": {"conferenceSolution": {"name": "Google Meet"}, "entryPoints": [
            {"entryPointType": "phone", "uri": "tel:+1"}, {"entryPointType": "video", "uri": "https://meet.google.com/abc"}
        ]},
        "attachments": [{"fileUrl": "https://drive/x", "title": "Agenda", "mimeType": "application/pdf"}]
    }));
    assert_eq!(w.oid, "");
    assert_eq!(w.remote_id.as_deref(), Some("primary/e1"));
    assert_eq!((w.kind.as_str(), w.subject.as_str(), w.body.as_str(), w.path.as_str()), ("event", "Standup", "notes", "me@example.com/"));
    let e = w.event.unwrap();
    assert_eq!(e.start, "2026-09-25T14:00:00-04:00[America/New_York]");
    assert_eq!(e.end, "2026-09-25T14:30:00-04:00[America/New_York]");
    assert_eq!(end, 1_790_361_000);
    assert_eq!(e.timezone.as_deref(), Some("America/New_York"));
    assert_eq!((e.status.as_str(), e.transparency.as_str(), e.visibility.as_str(), e.event_type.as_str()), ("tentative", "busy", "private", "focus_time"));
    assert_eq!(e.color.as_deref(), Some("5"));
    assert_eq!(e.organizer.unwrap().name.as_deref(), Some("Boss"));
    assert_eq!(e.attendees.len(), 2, "an attendee with no email is left out");
    assert!(e.attendees[0].is_self && e.attendees[0].response == "declined");
    assert_eq!(e.attendees[1].response, "needs_action");
    let c = e.conference.unwrap();
    assert_eq!((c.provider.as_str(), c.url.as_str()), ("Google Meet", "https://meet.google.com/abc"));
    assert_eq!(e.attachments[0].mime_type.as_deref(), Some("application/pdf"));
}

#[test]
fn an_all_day_event_is_two_dates_and_ends_at_midnight_in_the_calendars_zone() {
    let (w, end) = live(serde_json::json!({"id": "d1", "summary": "Offsite", "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"}}));
    let e = w.event.unwrap();
    assert_eq!((e.start.as_str(), e.end.as_str()), ("2026-09-25", "2026-09-26"));
    assert_eq!(end, 1_790_395_200, "2026-09-26T00:00-04:00");
    assert_eq!(e.timezone, None);
}

#[test]
fn a_timed_event_with_no_zone_of_its_own_takes_the_calendars() {
    let (w, _) = live(serde_json::json!({"id": "e2", "start": {"dateTime": "2026-09-25T18:00:00Z"}, "end": {"dateTime": "2026-09-25T19:00:00Z"}}));
    assert_eq!(w.event.unwrap().start, "2026-09-25T14:00:00-04:00[America/New_York]");
}

/// Google's absent defaults are confirmed, opaque and default; words dam has
/// no value for read as the default rather than failing the pull.
#[test]
fn absent_and_unknown_words_read_as_googles_defaults() {
    let (w, _) = live(serde_json::json!({"id": "e3", "eventType": "fromGmail", "visibility": "somethingNew", "transparency": "transparent",
        "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"}}));
    let e = w.event.unwrap();
    assert_eq!((e.status.as_str(), e.transparency.as_str(), e.visibility.as_str(), e.event_type.as_str()), ("confirmed", "free", "default", "default"));
    assert_eq!(w.subject, "");
}

#[test]
fn a_cancelled_item_is_its_remote_id_whatever_else_it_lacks() {
    let mapped = map_event(&listing(), &google(serde_json::json!({"id": "gone", "status": "cancelled"}))).unwrap();
    assert!(matches!(mapped, Mapped::Cancelled(id) if id == "primary/gone"));
}

#[test]
fn a_live_event_without_a_start_is_an_error_naming_the_calendar_and_the_event() {
    let err = map_event(&listing(), &google(serde_json::json!({"id": "broken", "end": {"date": "2026-09-26"}}))).unwrap_err();
    assert_eq!(err.to_string(), "calendar primary, event broken: start is missing");
}

#[test]
fn a_calendar_title_is_one_path_segment() {
    let mut l = listing();
    l.summary = Some("Work/Team".into());
    let (w, _) = match map_event(&l, &google(serde_json::json!({"id": "e", "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"}}))).unwrap() {
        Mapped::Live { object, end } => (*object, end),
        Mapped::Cancelled(_) => unreachable!(),
    };
    assert_eq!(w.path, "Work-Team/");
    l.summary = Some("  ".into());
    let Mapped::Live { object, .. } = map_event(&l, &google(serde_json::json!({"id": "e", "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"}}))).unwrap() else { unreachable!() };
    assert_eq!(object.path, "primary/");
}

#[test]
fn a_remote_id_splits_back_at_its_last_slash() {
    let id = remote_id("en.usa#holiday@group.v.calendar.google.com", "abc_20260925");
    assert_eq!(calendar_of(&id), Some("en.usa#holiday@group.v.calendar.google.com"));
}
```

- [ ] **Step 2: Write the failing pull tests**

```rust
// crates/dam-remote-gcal/tests/pull.rs
mod loopback;
mod support;

use std::collections::HashMap;

use dam_remote_gcal::calendar_api::CalendarApi;
use dam_remote_gcal::pull::{pull, window};
use dam_remote_gcal::{Endpoints, http_agent};
use loopback::Reply;

const NOW: &str = "2026-09-22T12:00:00Z";

fn api(base: &str) -> CalendarApi {
    CalendarApi::new(http_agent(), &Endpoints::loopback(base).unwrap(), "ya29.A".into())
}

fn page(summary: &str, items: serde_json::Value) -> Vec<Reply> {
    vec![Reply::json(200, serde_json::json!({"summary": summary, "timeZone": "UTC", "items": items}))]
}

fn event(id: &str) -> serde_json::Value {
    serde_json::json!({"id": id, "summary": id, "start": {"dateTime": "2026-09-23T10:00:00Z"}, "end": {"dateTime": "2026-09-23T11:00:00Z"}})
}

#[test]
fn the_window_is_a_week_back_and_ninety_days_ahead() {
    let w = window(NOW.parse().unwrap());
    assert_eq!(w.from.to_string(), "2026-09-15T12:00:00Z");
    assert_eq!(w.to.to_string(), "2026-12-21T12:00:00Z");
}

#[test]
fn every_calendar_is_pulled_and_cancelled_items_come_back_by_id() {
    let _guard = support::guard("every_calendar_is_pulled_and_cancelled_items_come_back_by_id");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", page("me@example.com", serde_json::json!([event("a"), {"id": "gone", "status": "cancelled"}])));
    routes.insert("GET /calendar/v3/calendars/team%40x/events", page("Team", serde_json::json!([event("b")])));
    let google = loopback::serve(routes);
    let response = pull(&api(&google.base), &["primary".into(), "team@x".into()], None, NOW.parse().unwrap()).unwrap();
    let ids: Vec<&str> = response.objects.iter().filter_map(|o| o.remote_id.as_deref()).collect();
    assert_eq!(ids, vec!["primary/a", "team@x/b"]);
    assert_eq!(response.objects[1].path, "Team/");
    assert_eq!(response.cancelled, vec!["primary/gone"]);
    assert!(response.removed.is_empty());
    assert!(response.sync.is_some());
}

#[test]
fn one_unreadable_event_fails_the_whole_pull_and_names_it() {
    let _guard = support::guard("one_unreadable_event_fails_the_whole_pull_and_names_it");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", page("me", serde_json::json!([event("a"), {"id": "broken"}])));
    let google = loopback::serve(routes);
    let said = pull(&api(&google.base), &["primary".into()], None, NOW.parse().unwrap()).unwrap_err().to_string();
    assert_eq!(said, "calendar primary, event broken: start is missing");
}

#[test]
fn one_calendar_that_fails_fails_the_pull() {
    let _guard = support::guard("one_calendar_that_fails_fails_the_pull");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", page("me", serde_json::json!([event("a")])));
    let google = loopback::serve(routes);
    let err = pull(&api(&google.base), &["primary".into(), "missing".into()], None, NOW.parse().unwrap()).unwrap_err();
    assert!(err.to_string().contains("404"), "{err}");
}
```

A calendar that fails fails the whole pull because a dropped calendar reads exactly like a clear
one: every meeting on it would silently stop counting as busy. The same holds for one event the
helper cannot read.

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p dam-remote-gcal`
Expected: unresolved `map`, `pull`.

- [ ] **Step 4: Implement**

```rust
// crates/dam-remote-gcal/src/map/time.rs
//! A Google start or end as the text dam reads (a date, or an RFC 9557
//! zoned time), with its epoch second.

use jiff::civil::Date;
use jiff::tz::TimeZone;

use crate::calendar_api::GoogleTime;

/// A zone Google names that this machine's time zone database does not know
/// reads as UTC: the instant is kept and only its written zone differs.
fn zone(name: Option<&str>) -> TimeZone {
    name.and_then(|n| TimeZone::get(n).ok()).unwrap_or(TimeZone::UTC)
}

pub(super) fn when(time: &GoogleTime, calendar_zone: Option<&str>) -> Result<(String, i64), &'static str> {
    if let Some(stated) = &time.date_time {
        let instant: jiff::Timestamp = stated.parse().map_err(|_| "a time is not RFC 3339")?;
        let zoned = instant.to_zoned(zone(time.time_zone.as_deref().or(calendar_zone)));
        return Ok((zoned.to_string(), instant.as_second()));
    }
    if let Some(stated) = &time.date {
        let day: Date = stated.parse().map_err(|_| "a date is not YYYY-MM-DD")?;
        let midnight = day.to_zoned(zone(calendar_zone)).map_err(|_| "a date is outside the calendar")?;
        return Ok((day.to_string(), midnight.timestamp().as_second()));
    }
    Err("a time names neither date nor dateTime")
}
```

```rust
// crates/dam-remote-gcal/src/map.rs
//! One Google event as the wire event dam reads, or, when Google cancelled
//! it, just its remote id.

mod time;

use std::fmt;

use dam_protocol::{WireAttachment, WireAttendee, WireConference, WireEvent, WireObject, WirePerson};

use crate::calendar_api::{GoogleEvent, Listing};

/// The live object is boxed because it is hundreds of bytes and the
/// cancelled id is one string.
#[derive(Debug)]
pub(crate) enum Mapped {
    Live { object: Box<WireObject>, end: i64 },
    Cancelled(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct MapError {
    pub calendar: String,
    pub event_id: String,
    pub what: &'static str,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "calendar {}, event {}: {}", self.calendar, self.event_id, self.what)
    }
}

impl std::error::Error for MapError {}

pub fn remote_id(calendar: &str, event_id: &str) -> String {
    format!("{calendar}/{event_id}")
}

pub(crate) fn calendar_of(remote_id: &str) -> Option<&str> {
    remote_id.rsplit_once('/').map(|(calendar, _)| calendar)
}

/// A title as exactly one path segment: `/` cannot add a level, and a title
/// that names nothing falls back to the calendar's id.
fn segment(summary: Option<&str>, calendar: &str) -> String {
    let clean = |s: &str| s.replace('/', "-");
    match summary.map(clean) {
        Some(s) if !s.trim().is_empty() && s != "." && s != ".." => format!("{s}/"),
        _ => format!("{}/", clean(calendar)),
    }
}

pub(crate) fn map_event(listing: &Listing, event: &GoogleEvent) -> Result<Mapped, MapError> {
    let id = remote_id(&listing.calendar, &event.id);
    if event.status.as_deref() == Some("cancelled") {
        return Ok(Mapped::Cancelled(id));
    }
    let fail = |what| MapError { calendar: listing.calendar.clone(), event_id: event.id.clone(), what };
    let zone = listing.time_zone.as_deref();
    let (start, _) = time::when(event.start.as_ref().ok_or_else(|| fail("start is missing"))?, zone).map_err(fail)?;
    let (end_text, end) = time::when(event.end.as_ref().ok_or_else(|| fail("end is missing"))?, zone).map_err(fail)?;
    let wire_event = WireEvent {
        start,
        end: end_text,
        timezone: event.start.as_ref().and_then(|s| s.time_zone.clone()),
        location: event.location.clone(),
        attendees: event.attendees.iter().filter_map(|a| Some(WireAttendee {
            email: a.email.clone()?,
            response: response(a.response_status.as_deref()).into(),
            is_self: a.is_self,
        })).collect(),
        status: if event.status.as_deref() == Some("tentative") { "tentative" } else { "confirmed" }.into(),
        transparency: if event.transparency.as_deref() == Some("transparent") { "free" } else { "busy" }.into(),
        visibility: visibility(event.visibility.as_deref()).into(),
        event_type: event_type(event.event_type.as_deref()).into(),
        color: event.color_id.clone(),
        organizer: event.organizer.as_ref().and_then(|o| Some(WirePerson { email: o.email.clone()?, name: o.display_name.clone() })),
        conference: event.conference_data.as_ref().and_then(|c| {
            let video = c.entry_points.iter().find(|p| p.entry_point_type.as_deref() == Some("video"))?;
            Some(WireConference {
                provider: c.conference_solution.as_ref().and_then(|s| s.name.clone()).unwrap_or_default(),
                url: video.uri.clone()?,
            })
        }),
        attachments: event.attachments.iter().map(|a| WireAttachment {
            url: a.file_url.clone(),
            title: a.title.clone().unwrap_or_default(),
            mime_type: a.mime_type.clone(),
        }).collect(),
    };
    let object = WireObject {
        oid: String::new(),
        remote_id: Some(id),
        kind: "event".into(),
        subject: event.summary.clone().unwrap_or_default(),
        body: event.description.clone().unwrap_or_default(),
        path: segment(listing.summary.as_deref(), &listing.calendar),
        labels: vec![],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: None,
        event: Some(wire_event),
    };
    Ok(Mapped::Live { object: Box::new(object), end })
}

fn response(google: Option<&str>) -> &'static str {
    match google {
        Some("accepted") => "accepted",
        Some("declined") => "declined",
        Some("tentative") => "tentative",
        _ => "needs_action",
    }
}

fn visibility(google: Option<&str>) -> &'static str {
    match google {
        Some("public") => "public",
        Some("private") => "private",
        Some("confidential") => "confidential",
        _ => "default",
    }
}

/// `fromGmail`, and any type Google adds later, read as `default`: dam has
/// no value for them, and one unknown word must not fail the pull.
fn event_type(google: Option<&str>) -> &'static str {
    match google {
        Some("focusTime") => "focus_time",
        Some("outOfOffice") => "out_of_office",
        Some("workingLocation") => "working_location",
        Some("birthday") => "birthday",
        _ => "default",
    }
}

#[cfg(test)]
mod tests;
```

If `map.rs` passes 250 implementation lines after `rustfmt`, move `response`, `visibility` and
`event_type` into `src/map/words.rs` as `pub(super)` functions.

```rust
// crates/dam-remote-gcal/src/pull.rs
//! One pull: every configured calendar over one window, every live event as
//! an object and every cancelled one by id.

use std::fmt;

use dam_protocol::PullResponse;
use jiff::{SignedDuration, Timestamp};

use crate::ApiError;
use crate::calendar_api::{CalendarApi, Window};
use crate::map::{MapError, Mapped, map_event};

/// A week back keeps late edits to recent events landing.
pub const LOOKBACK_DAYS: i64 = 7;
/// Ninety days ahead is what dam's store holds of the future.
pub const HORIZON_DAYS: i64 = 90;

pub fn window(now: Timestamp) -> Window {
    let day = SignedDuration::from_hours(24);
    Window {
        from: now.saturating_sub(day * LOOKBACK_DAYS as i32).unwrap_or(now),
        to: now.saturating_add(day * HORIZON_DAYS as i32).unwrap_or(now),
    }
}

#[derive(Debug)]
pub enum PullError {
    Api(ApiError),
    Map(MapError),
}

impl fmt::Display for PullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PullError::Api(e) => e.fmt(f),
            PullError::Map(e) => e.fmt(f),
        }
    }
}

pub fn pull(api: &CalendarApi, calendars: &[String], since: Option<&str>, now: Timestamp) -> Result<PullResponse, PullError> {
    let window = window(now);
    let mut response = PullResponse::default();
    for calendar in calendars {
        let listing = api.events(calendar, &window).map_err(PullError::Api)?;
        for event in &listing.items {
            match map_event(&listing, event).map_err(PullError::Map)? {
                Mapped::Live { object, .. } => response.objects.push(*object),
                Mapped::Cancelled(id) => response.cancelled.push(id),
            }
        }
    }
    let _ = since;
    response.sync = Some(String::new());
    Ok(response)
}
```

The `since` parameter and the empty token are what Task 11 fills in, so Task 11 changes only this
function's body. `lib.rs` gains `pub mod pull; pub(crate) mod map; pub use map::{MapError,
remote_id};`. The `jiff` calls were read from jiff 0.2.37, the version `Cargo.lock` pins:
`SignedDuration::from_hours` is a `const fn`, `SignedDuration` multiplies by an `i32`, and
`Timestamp::saturating_add` and `saturating_sub` return a `Result`.

- [ ] **Step 5: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 6: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): pull every configured calendar's window, cancelled items by id"
```

---

### Task 11: Events that vanish from the window

PR 5. An instance moved past the horizon, and a deleted event Google has since purged, stop
appearing with no cancelled item left behind, and dam would keep the old time as busy. The helper
remembers, in the opaque `sync` token dam hands back as `since`, the remote id and end second of
every live event it reported. An id it reported last time, whose calendar is still configured, whose
end is still inside the window, and which this pull neither lists nor cancels, is reported as
cancelled. If the event comes back into the window later, it arrives whole and moves back to
confirmed at its new time, because dam still tracks it under the same id.

**Files:**
- Create: `crates/dam-remote-gcal/src/memory.rs`
- Modify: `crates/dam-remote-gcal/src/pull.rs`, `src/lib.rs`
- Modify: `crates/dam-remote-gcal/tests/pull.rs`

**Interfaces:**
- Consumes: `calendar_of` (Task 10).
- Produces:

```rust
// memory.rs
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Memory { pub(crate) reported: std::collections::BTreeMap<String, i64> }   // remote id to end second
impl Memory {
    pub(crate) fn read(since: Option<&str>) -> Memory;    // an absent or unreadable token is an empty memory
    pub(crate) fn token(&self) -> String;
    pub(crate) fn vanished(&self, seen: &std::collections::BTreeSet<String>, calendars: &[String], window_start: i64) -> Vec<String>;
}
```

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-remote-gcal/src/memory.rs, at the bottom
#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn memory(entries: &[(&str, i64)]) -> Memory {
        Memory { reported: entries.iter().map(|(id, end)| (id.to_string(), *end)).collect() }
    }

    #[test]
    fn an_absent_or_unreadable_token_is_an_empty_memory() {
        assert_eq!(Memory::read(None), Memory::default());
        assert_eq!(Memory::read(Some("not json")), Memory::default());
        let m = memory(&[("primary/a", 10)]);
        assert_eq!(Memory::read(Some(&m.token())), m);
    }

    #[test]
    fn only_an_unseen_id_still_in_the_window_on_a_configured_calendar_has_vanished() {
        let m = memory(&[("primary/seen", 500), ("primary/moved", 500), ("primary/aged", 100), ("dropped@x/c", 500)]);
        let seen: BTreeSet<String> = ["primary/seen".to_string()].into();
        assert_eq!(m.vanished(&seen, &["primary".into()], 200), vec!["primary/moved"]);
    }
}
```

Append to `tests/pull.rs`:

```rust
#[test]
fn an_event_that_left_the_window_early_comes_back_cancelled_on_the_next_pull() {
    let _guard = support::guard("an_event_that_left_the_window_early_comes_back_cancelled_on_the_next_pull");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", vec![
        Reply::json(200, serde_json::json!({"summary": "me", "timeZone": "UTC", "items": [event("stays"), event("moves")]})),
        Reply::json(200, serde_json::json!({"summary": "me", "timeZone": "UTC", "items": [event("stays")]})),
    ]);
    let google = loopback::serve(routes);
    let first = pull(&api(&google.base), &["primary".into()], None, NOW.parse().unwrap()).unwrap();
    assert!(first.cancelled.is_empty());
    let second = pull(&api(&google.base), &["primary".into()], first.sync.as_deref(), NOW.parse().unwrap()).unwrap();
    assert_eq!(second.cancelled, vec!["primary/moves"]);
    let third = pull(&api(&google.base), &["primary".into()], second.sync.as_deref(), NOW.parse().unwrap()).unwrap();
    assert!(third.cancelled.is_empty(), "reported once, then forgotten");
}

#[test]
fn a_calendar_taken_off_the_address_is_forgotten_rather_than_cancelled() {
    let _guard = support::guard("a_calendar_taken_off_the_address_is_forgotten_rather_than_cancelled");
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", page("me", serde_json::json!([event("a")])));
    routes.insert("GET /calendar/v3/calendars/team%40x/events", page("Team", serde_json::json!([event("b")])));
    let google = loopback::serve(routes);
    let first = pull(&api(&google.base), &["primary".into(), "team@x".into()], None, NOW.parse().unwrap()).unwrap();
    let second = pull(&api(&google.base), &["primary".into()], first.sync.as_deref(), NOW.parse().unwrap()).unwrap();
    assert!(second.cancelled.is_empty());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-remote-gcal`
Expected: unresolved `Memory`; the two pull tests fail on an empty `cancelled`.

- [ ] **Step 3: Implement**

```rust
// crates/dam-remote-gcal/src/memory.rs
//! What the last pull reported, carried in the sync token dam hands back:
//! each live event's remote id and end second, nothing of its text.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::map::calendar_of;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Memory {
    pub(crate) reported: BTreeMap<String, i64>,
}

impl Memory {
    /// An unreadable token costs one pull's vanish check and nothing else,
    /// so it reads as no memory rather than failing the pull.
    pub(crate) fn read(since: Option<&str>) -> Memory {
        since.and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default()
    }

    pub(crate) fn token(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub(crate) fn vanished(&self, seen: &BTreeSet<String>, calendars: &[String], window_start: i64) -> Vec<String> {
        self.reported
            .iter()
            .filter(|(id, end)| {
                !seen.contains(*id)
                    && **end > window_start
                    && calendar_of(id).is_some_and(|c| calendars.iter().any(|k| k == c))
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
}
```

`pull` becomes:

```rust
pub fn pull(api: &CalendarApi, calendars: &[String], since: Option<&str>, now: Timestamp) -> Result<PullResponse, PullError> {
    let window = window(now);
    let previous = Memory::read(since);
    let mut next = Memory::default();
    let mut seen = BTreeSet::new();
    let mut response = PullResponse::default();
    for calendar in calendars {
        let listing = api.events(calendar, &window).map_err(PullError::Api)?;
        for event in &listing.items {
            match map_event(&listing, event).map_err(PullError::Map)? {
                Mapped::Live { object, end } => {
                    if let Some(id) = &object.remote_id {
                        seen.insert(id.clone());
                        next.reported.insert(id.clone(), end);
                    }
                    response.objects.push(*object);
                }
                Mapped::Cancelled(id) => {
                    seen.insert(id.clone());
                    response.cancelled.push(id);
                }
            }
        }
    }
    response.cancelled.extend(previous.vanished(&seen, calendars, window.from.as_second()));
    response.sync = Some(next.token());
    Ok(response)
}
```

`lib.rs` gains `mod memory;`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-remote-gcal && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 5: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): report an event that vanished from the window as cancelled"
```

---

### Task 12: The protocol binary, and push refused

PR 5. A push is answered per mutation, each refused with one sentence, so every other remote's push
goes ahead and `dam status` says, against each event, why it did not reach Google. The helper reads
no credential and sends no request on a push. dam's existing retry rule then resends a refused
mutation on the next push, where it is refused again with the same sentence.

**Files:**
- Create: `crates/dam-remote-gcal/src/push.rs`, `src/main.rs`
- Create: `crates/dam-remote-gcal/tests/protocol.rs`
- Modify: `crates/dam-remote-gcal/Cargo.toml` (`[[bin]] name = "dam-remote-gcal"`), `src/lib.rs`
- Modify: `README.md` (the gcal remote's config)

**Interfaces:**
- Consumes: everything above; `dam_protocol::{Request, Response, read_line, write_line, Mutation, MutationResult, PushResponse}`.
- Produces:

```rust
// push.rs
pub const READ_ONLY: &str;   // the refusal sentence
pub fn refuse(mutations: &[Mutation]) -> PushResponse;   // one ok:false result per mutation, in order
```

and the binary `dam-remote-gcal <remote> <address>`: exactly two arguments, else usage on standard
error and exit 2.

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-remote-gcal/src/push.rs, at the bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mutation_is_refused_with_the_sentence_in_order() {
        let mutation = |oid: &str, op: &str| Mutation {
            op: op.into(), oid: oid.into(), idempotency_key: "k".into(), remote_id: None, object: None, fields: vec![],
        };
        let answer = refuse(&[mutation("a", "create"), mutation("b", "delete")]);
        assert_eq!(answer.results.iter().map(|r| r.oid.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert!(answer.results.iter().all(|r| !r.ok && r.why.as_deref() == Some(READ_ONLY) && r.remote_id.is_none()));
        assert!(READ_ONLY.contains("read-only"));
    }
}
```

```rust
// crates/dam-remote-gcal/tests/protocol.rs
mod loopback;
mod support;

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use dam_protocol::Response;
use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";
const ACCESS: &str = "ya29.SUPERSECRETACCESS";

/// The helper as dam runs it, in a temporary home, with the loopback seam.
fn helper(home: &std::path::Path, args: &[&str], base: &str, credentials: bool) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_dam-remote-gcal"));
    command
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_HOME", home.join("data"))
        .env("DAM_GCAL_BASE_URL", base)
        .env_remove("DAM_GCAL_CLIENT_ID")
        .env_remove("DAM_GCAL_CLIENT_SECRET")
        .env_remove("DAM_GCAL_REFRESH_TOKEN")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if credentials {
        command.env("DAM_GCAL_CLIENT_ID", "123.apps").env("DAM_GCAL_CLIENT_SECRET", CLIENT_SECRET).env("DAM_GCAL_REFRESH_TOKEN", REFRESH);
    }
    command
}

fn converse(mut command: Command, requests: &str) -> (Vec<Response>, String) {
    let mut child = command.spawn().unwrap();
    child.stdin.take().unwrap().write_all(requests.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    let responses = String::from_utf8(out.stdout).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    (responses, String::from_utf8_lossy(&out.stderr).into_owned())
}

fn google() -> loopback::Loopback {
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![Reply::json(200, serde_json::json!({"access_token": ACCESS, "expires_in": 3599}))]);
    routes.insert("GET /calendar/v3/calendars/primary/events", vec![Reply::json(200, serde_json::json!({"summary": "me", "timeZone": "UTC", "items": [
        {"id": "a", "summary": "Standup", "start": {"dateTime": "2026-09-23T10:00:00Z"}, "end": {"dateTime": "2026-09-23T11:00:00Z"}}
    ]}))]);
    loopback::serve(routes)
}

#[test]
fn capabilities_need_no_credentials_and_a_pull_names_the_missing_variable_only() {
    let _guard = support::guard("capabilities_need_no_credentials_and_a_pull_names_the_missing_variable_only");
    let home = tempfile::tempdir().unwrap();
    let google = google();
    let (responses, _) = converse(helper(home.path(), &["gcal", ""], &google.base, false), "{\"cmd\":\"capabilities\"}\n{\"cmd\":\"pull\",\"since\":null}\n");
    assert!(matches!(&responses[0], Response::Capabilities(c) if c.kinds == vec!["event"]));
    let Response::Error { error } = &responses[1] else { panic!("{:?}", responses[1]) };
    assert!(error.contains("DAM_GCAL_CLIENT_ID"), "{error}");
    assert!(google.seen().is_empty());
}

#[test]
fn a_pull_reaches_the_token_endpoint_then_the_calendar_and_answers_dams_document() {
    let _guard = support::guard("a_pull_reaches_the_token_endpoint_then_the_calendar_and_answers_dams_document");
    let home = tempfile::tempdir().unwrap();
    let google = google();
    let (responses, stderr) = converse(helper(home.path(), &["gcal", ""], &google.base, true), "{\"cmd\":\"pull\",\"since\":null}\n");
    let Response::Pull(pull) = &responses[0] else { panic!("{:?}", responses[0]) };
    assert_eq!(pull.objects[0].remote_id.as_deref(), Some("primary/a"));
    let seen = google.seen();
    assert_eq!((seen[0].path.as_str(), seen[1].path.as_str()), ("/token", "/calendar/v3/calendars/primary/events"));
    assert_eq!(seen[1].authorization.as_deref(), Some(format!("Bearer {ACCESS}").as_str()));
    for secret in [CLIENT_SECRET, REFRESH, ACCESS] {
        assert!(!stderr.contains(secret));
    }
}

#[test]
fn a_remote_under_another_name_reads_its_own_credentials() {
    let _guard = support::guard("a_remote_under_another_name_reads_its_own_credentials");
    let home = tempfile::tempdir().unwrap();
    let google = google();
    let mut command = helper(home.path(), &["work", "primary"], &google.base, false);
    command.env("DAM_WORK_CLIENT_ID", "id").env("DAM_WORK_CLIENT_SECRET", CLIENT_SECRET).env("DAM_WORK_REFRESH_TOKEN", REFRESH);
    let (responses, _) = converse(command, "{\"cmd\":\"pull\",\"since\":null}\n");
    assert!(matches!(&responses[0], Response::Pull(_)), "{:?}", responses[0]);
}

#[test]
fn a_push_is_refused_per_mutation_without_a_credential_or_a_request() {
    let _guard = support::guard("a_push_is_refused_per_mutation_without_a_credential_or_a_request");
    let home = tempfile::tempdir().unwrap();
    let google = google();
    let push = r#"{"cmd":"push","mutations":[{"op":"update","oid":"0101010101010101010101010101010101010101","idempotency_key":"k","remote_id":"primary/a","fields":["subject"]}]}"#;
    let (responses, _) = converse(helper(home.path(), &["gcal", ""], &google.base, false), &format!("{push}\n"));
    let Response::Push(answer) = &responses[0] else { panic!("{:?}", responses[0]) };
    assert!(!answer.results[0].ok);
    assert!(answer.results[0].why.as_deref().unwrap().contains("read-only"));
    assert!(google.seen().is_empty(), "a push sent a request");
}

#[test]
fn any_invocation_but_remote_and_address_is_a_usage_error() {
    let _guard = support::guard("any_invocation_but_remote_and_address_is_a_usage_error");
    let home = tempfile::tempdir().unwrap();
    for args in [&[][..], &["gcal"][..], &["gcal", "", "extra"][..]] {
        let out = helper(home.path(), args, "http://127.0.0.1:1", false).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-remote-gcal`
Expected: `CARGO_BIN_EXE_dam-remote-gcal` not defined; unresolved `push`.

- [ ] **Step 3: Implement**

```rust
// crates/dam-remote-gcal/src/push.rs
//! Every mutation refused, by name. This helper never writes to Google.

use dam_protocol::{Mutation, MutationResult, PushResponse};

pub const READ_ONLY: &str = "Google Calendar is read-only through dam-remote-gcal: it pulls events \
and never creates, changes or deletes one, so this change stays in dam";

pub fn refuse(mutations: &[Mutation]) -> PushResponse {
    PushResponse {
        results: mutations
            .iter()
            .map(|m| MutationResult { oid: m.oid.clone(), ok: false, remote_id: None, why: Some(READ_ONLY.into()) })
            .collect(),
    }
}
```

```toml
# crates/dam-remote-gcal/Cargo.toml, beside the other [[bin]]
[[bin]]
name = "dam-remote-gcal"
path = "src/main.rs"
```

```rust
// crates/dam-remote-gcal/src/main.rs
use std::io::{self, BufRead, Write};

use dam_protocol::{Request, Response, read_line, write_line};
use dam_remote_gcal::access::access_token;
use dam_remote_gcal::address::calendars;
use dam_remote_gcal::calendar_api::CalendarApi;
use dam_remote_gcal::{Credentials, Endpoints, capabilities, http_agent, pull, push};

const USAGE: &str = "usage: dam-remote-gcal <remote> <address>\n  dam runs this; to sign in, run dam-gcal-sign-in";

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [remote, address] = arguments.as_slice() else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    let stdin = io::stdin();
    run(&mut stdin.lock(), &mut io::stdout().lock(), remote, address);
}

/// One response per request line. A line that does not decode answers an
/// error and the loop goes on; any other read failure ends it, since
/// retrying a broken pipe would spin without ever reaching the end.
fn run(input: &mut impl BufRead, out: &mut impl Write, remote: &str, address: &str) {
    loop {
        let request: Option<Request> = match read_line(input) {
            Ok(r) => r,
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                let _ = write_line(out, &Response::Error { error: format!("cannot read the request: {e}") });
                continue;
            }
            Err(_) => break,
        };
        let Some(request) = request else { break };
        if write_line(out, &answer(request, remote, address)).is_err() {
            break;
        }
    }
}

fn answer(request: Request, remote: &str, address: &str) -> Response {
    match request {
        Request::Capabilities => Response::Capabilities(capabilities::capabilities()),
        Request::Push { mutations } => Response::Push(push::refuse(&mutations)),
        Request::Pull { since } => match pull_now(remote, address, since.as_deref()) {
            Ok(pulled) => Response::Pull(pulled),
            Err(error) => Response::Error { error },
        },
    }
}

fn pull_now(remote: &str, address: &str, since: Option<&str>) -> Result<dam_protocol::PullResponse, String> {
    let credentials = Credentials::from_env(remote)?;
    let endpoints = Endpoints::from_env()?;
    let agent = http_agent();
    let access = access_token(&agent, &endpoints, &credentials).map_err(|e| e.to_string())?;
    let api = CalendarApi::new(agent, &endpoints, access);
    pull::pull(&api, &calendars(address), since, jiff::Timestamp::now()).map_err(|e| e.to_string())
}
```

Copy the `a_non_decode_read_error_ends_the_loop_without_spinning_forever` unit test and its
`FailingReader` from `crates/dam-remote-todoist/src/main.rs`, calling `run(&mut input, &mut out,
"gcal", "")`.

README, in the "Google Calendar" section after the sign-in:

```markdown
Then the remote. The address after `gcal::` names the calendars to read, comma-separated; empty
means your primary calendar:

    dam remote add gcal gcal::primary

    [remote.gcal]
    url = "gcal::primary"
    client_id_command = ["security", "find-generic-password", "-w", "-s", "Google dam client id"]
    client_secret_command = ["security", "find-generic-password", "-w", "-s", "Google dam client"]
    refresh_token_command = ["security", "find-generic-password", "-w", "-s", "Google dam refresh"]
    stale = "5m"

A pull reads the week behind and the ninety days ahead, recurring events as their instances. The
helper is read-only: `dam push` answers each event change sent to it with a refusal that `dam status`
shows, and nothing reaches Google.
```

- [ ] **Step 4: Run to verify pass**

Run: `just gates`
Expected: clean; every test inside a second.

- [ ] **Step 5: Commit, and apply the amendment once approved**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-remote-gcal README.md
SKIP_AI_COMMIT=1 git commit -m "feat(gcal): the read-only protocol helper, refusing every push by name"
```

Once Amendment A4 is approved, apply the rest of it (the helper paragraph and the config example's
comment) in its own commit, `docs(spec): dam-remote-gcal pulls read-only`. Pending: stop and ask.

---

### Task 13: When an event holds time

PR 6. Needs PR 3 (`Attendee.is_self`).

**Files:**
- Modify: `crates/dam-domain/src/when/mod.rs` (`When::instant`)
- Modify: `crates/dam-domain/src/object/event.rs` (`Event::span`, `Event::holds_time`)
- Create: `crates/dam-domain/src/object/event/tests.rs`

**Interfaces:**
- Produces:

```rust
impl When {
    /// The instant this names: itself for a zoned time, the first instant of the day in `zone` for a date.
    pub fn instant(&self, zone: &jiff::tz::TimeZone) -> Option<Timestamp>;
}
impl Event {
    /// Start and end as instants, the end exclusive. A whole day runs midnight to midnight in `zone`,
    /// because an all-day event is a date wherever the person is rather than a fixed span.
    pub fn span(&self, zone: &jiff::tz::TimeZone) -> Option<(Timestamp, Timestamp)>;
    /// Whether the event holds its calendar owner's time: not cancelled, shown busy, and not declined
    /// by the calendar's own attendee. Tentative and unanswered still hold it.
    pub fn holds_time(&self) -> bool;
}
```

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-domain/src/object/event/tests.rs
use jiff::civil::date;
use jiff::tz::TimeZone;

use super::*;
use crate::{Oid, When};

fn oid() -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(7))
}

fn timed() -> Event {
    let start = "2026-09-25T14:00:00-04:00[America/New_York]".parse().unwrap();
    let end = "2026-09-25T14:30:00-04:00[America/New_York]".parse().unwrap();
    Event::new(oid(), "standup", When::At(start), When::At(end))
}

fn attendee(is_self: bool, response: ResponseStatus) -> Attendee {
    Attendee { email: "x@example.com".into(), response, is_self }
}

#[test]
fn a_timed_span_is_its_own_two_instants_whatever_zone_is_given() {
    let (start, end) = timed().span(&TimeZone::UTC).unwrap();
    assert_eq!((start.as_second(), end.as_second()), (1_790_359_200, 1_790_361_000));
}

#[test]
fn an_all_day_span_runs_midnight_to_midnight_in_the_zone_given() {
    let day = Event::new(oid(), "offsite", When::Day(date(2026, 9, 25)), When::Day(date(2026, 9, 26)));
    let (start, end) = day.span(&TimeZone::fixed(jiff::tz::offset(-4))).unwrap();
    assert_eq!((start.as_second(), end.as_second()), (1_790_308_800, 1_790_395_200));
}

#[test]
fn an_all_day_span_on_a_spring_forward_day_is_twenty_three_hours() {
    let zone = TimeZone::posix("EST5EDT,M3.2.0,M11.1.0").unwrap();
    let day = Event::new(oid(), "dst", When::Day(date(2026, 3, 8)), When::Day(date(2026, 3, 9)));
    let (start, end) = day.span(&zone).unwrap();
    assert_eq!(end.as_second() - start.as_second(), 23 * 3600);
}

#[test]
fn a_confirmed_busy_event_holds_time_and_so_does_a_tentative_one() {
    assert!(timed().holds_time());
    let mut tentative = timed();
    tentative.status = EventStatus::Tentative;
    tentative.attendees = vec![attendee(true, ResponseStatus::NeedsAction)];
    assert!(tentative.holds_time());
}

#[test]
fn a_cancelled_or_free_event_holds_no_time() {
    let mut cancelled = timed();
    cancelled.status = EventStatus::Cancelled;
    assert!(!cancelled.holds_time());
    let mut free = timed();
    free.transparency = Transparency::Free;
    assert!(!free.holds_time());
}

#[test]
fn an_event_the_calendars_own_attendee_declined_holds_no_time() {
    let mut declined = timed();
    declined.attendees = vec![attendee(false, ResponseStatus::Accepted), attendee(true, ResponseStatus::Declined)];
    assert!(!declined.holds_time());
    let mut someone_else = timed();
    someone_else.attendees = vec![attendee(false, ResponseStatus::Declined)];
    assert!(someone_else.holds_time(), "only the owner's answer counts");
}
```

Add `#[cfg(test)] mod tests;` at the bottom of `event.rs`, and to `when/mod.rs`'s tests module:

```rust
#[test]
fn a_date_is_its_first_instant_in_the_zone_and_a_zoned_time_is_itself() {
    let zone = jiff::tz::TimeZone::fixed(jiff::tz::offset(-4));
    assert_eq!(When::Day(jiff::civil::date(2026, 9, 25)).instant(&zone).unwrap().as_second(), 1_790_308_800);
    let at: jiff::Zoned = "2026-09-25T14:00:00-04:00[America/New_York]".parse().unwrap();
    assert_eq!(When::At(at.clone()).instant(&jiff::tz::TimeZone::UTC), Some(at.timestamp()));
}
```

The zoned strings parse with jiff's bundled or system tzdb in a test; the zones the domain code
itself uses are always passed in.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-domain`
Expected: no method `instant`, `span`, `holds_time`.

- [ ] **Step 3: Implement**

```rust
// crates/dam-domain/src/when/mod.rs, in impl When
pub fn instant(&self, zone: &jiff::tz::TimeZone) -> Option<Timestamp> {
    match self {
        When::At(z) => Some(z.timestamp()),
        When::Day(d) => d.to_zoned(zone.clone()).ok().map(|z| z.timestamp()),
    }
}
```

```rust
// crates/dam-domain/src/object/event.rs, a new impl block
impl Event {
    pub fn span(&self, zone: &jiff::tz::TimeZone) -> Option<(crate::Timestamp, crate::Timestamp)> {
        Some((self.start.instant(zone)?, self.end.instant(zone)?))
    }

    pub fn holds_time(&self) -> bool {
        self.status != EventStatus::Cancelled
            && self.transparency == Transparency::Busy
            && !self.attendees.iter().any(|a| a.is_self && a.response == ResponseStatus::Declined)
    }
}
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-domain && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 5: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-domain
SKIP_AI_COMMIT=1 git commit -m "feat(event): an event's span in instants, and whether it holds time"
```

---

### Task 14: The agenda use case

PR 6.

**Files:**
- Create: `crates/dam-application/src/use_cases/agenda.rs`
- Modify: `crates/dam-application/src/use_cases/mod.rs`, `crates/dam-application/src/lib.rs`

**Interfaces:**
- Consumes: `list` (existing), `Event::span`, `Event::holds_time` (Task 13).
- Produces:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window { pub from: Timestamp, pub to: Timestamp }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheduled { pub event: Event, pub start: Timestamp, pub end: Timestamp, pub busy: bool }
/// Every event the query admits whose span overlaps the window (start before `to`, end after
/// `from`), busy or not, ordered by start and then oid. Tasks are never listed.
pub fn agenda(objects: &dyn ObjectRepository, clock: &dyn Clock, config: &Config, query: Option<&str>, window: Window, zone: &jiff::tz::TimeZone) -> Result<Vec<Scheduled>, UseCaseError>;
```

Exported from `lib.rs` as `pub use use_cases::agenda::{Scheduled, Window, agenda};`.

- [ ] **Step 1: Write the failing tests**

```rust
// crates/dam-application/src/use_cases/agenda.rs, at the bottom
#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use jiff::tz::TimeZone;

    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FixedClock, MemoryStore, oid};
    use dam_domain::{EventStatus, Object, Path, Task, When};

    fn at(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    fn event(n: u8, subject: &str, start: &str, end: &str) -> Event {
        let zoned = |t: &str| When::At(at(t).to_zoned(TimeZone::UTC));
        Event::new(oid(n), subject, zoned(start), zoned(end))
    }

    fn run(store: &MemoryStore, query: Option<&str>, from: &str, to: &str) -> Vec<Scheduled> {
        agenda(store, &FixedClock(date(2026, 9, 25)), &Config::default(), query, Window { from: at(from), to: at(to) }, &TimeZone::UTC).unwrap()
    }

    fn subjects(found: &[Scheduled]) -> Vec<&str> {
        found.iter().map(|s| s.event.base.subject.as_str()).collect()
    }

    #[test]
    fn an_event_in_progress_when_the_window_opens_is_listed() {
        let store = MemoryStore::new();
        store.put(&Object::Event(event(1, "running", "2026-09-25T09:00:00Z", "2026-09-25T11:00:00Z"))).unwrap();
        let found = run(&store, None, "2026-09-25T10:00:00Z", "2026-09-26T10:00:00Z");
        assert_eq!(subjects(&found), vec!["running"]);
        assert_eq!((found[0].start, found[0].end, found[0].busy), (at("2026-09-25T09:00:00Z"), at("2026-09-25T11:00:00Z"), true));
    }

    #[test]
    fn the_window_is_half_open_on_both_sides() {
        let store = MemoryStore::new();
        store.put(&Object::Event(event(1, "ended at open", "2026-09-25T08:00:00Z", "2026-09-25T10:00:00Z"))).unwrap();
        store.put(&Object::Event(event(2, "starts at close", "2026-09-26T10:00:00Z", "2026-09-26T11:00:00Z"))).unwrap();
        assert!(run(&store, None, "2026-09-25T10:00:00Z", "2026-09-26T10:00:00Z").is_empty());
    }

    #[test]
    fn events_are_ordered_by_start_and_a_cancelled_one_is_listed_as_not_busy() {
        let store = MemoryStore::new();
        let mut gone = event(1, "gone", "2026-09-25T15:00:00Z", "2026-09-25T16:00:00Z");
        gone.status = EventStatus::Cancelled;
        store.put(&Object::Event(gone)).unwrap();
        store.put(&Object::Event(event(2, "first", "2026-09-25T12:00:00Z", "2026-09-25T13:00:00Z"))).unwrap();
        let found = run(&store, None, "2026-09-25T00:00:00Z", "2026-09-26T00:00:00Z");
        assert_eq!(subjects(&found), vec!["first", "gone"]);
        assert!(found[0].busy && !found[1].busy);
    }

    #[test]
    fn tasks_are_never_listed_and_a_query_narrows_the_events() {
        let store = MemoryStore::new();
        let mut due = Task::new(oid(3), "a task due now");
        due.due = Some(When::Day(date(2026, 9, 25)));
        store.put(&Object::Task(due)).unwrap();
        let mut work = event(4, "work", "2026-09-25T12:00:00Z", "2026-09-25T13:00:00Z");
        work.base.path = Path::parse("work").unwrap();
        store.put(&Object::Event(work)).unwrap();
        store.put(&Object::Event(event(5, "home", "2026-09-25T12:00:00Z", "2026-09-25T13:00:00Z"))).unwrap();
        assert_eq!(subjects(&run(&store, None, "2026-09-25T00:00:00Z", "2026-09-26T00:00:00Z")), vec!["work", "home"]);
        assert_eq!(subjects(&run(&store, Some("path:work/"), "2026-09-25T00:00:00Z", "2026-09-26T00:00:00Z")), vec!["work"]);
    }
}
```

Two events at the same start order by oid, and `oid(4)` sorts before `oid(5)`, which is why the
unnarrowed list reads `["work", "home"]`.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p dam-application agenda`
Expected: unresolved `agenda`, `Window`, `Scheduled`.

- [ ] **Step 3: Implement**

```rust
// crates/dam-application/src/use_cases/agenda.rs
//! Events as intervals over a window, each with whether it holds time: the
//! reading a scheduler takes busy times from.

use dam_domain::{Event, Object, Timestamp};

use crate::config::Config;
use crate::errors::UseCaseError;
use crate::ports::{Clock, ObjectRepository};
use crate::use_cases::list::list;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: Timestamp,
    pub to: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheduled {
    pub event: Event,
    pub start: Timestamp,
    pub end: Timestamp,
    pub busy: bool,
}

pub fn agenda(objects: &dyn ObjectRepository, clock: &dyn Clock, config: &Config, query: Option<&str>, window: Window, zone: &jiff::tz::TimeZone) -> Result<Vec<Scheduled>, UseCaseError> {
    let mut found: Vec<Scheduled> = list(objects, clock, config, query)?
        .into_iter()
        .filter_map(|o| match o {
            Object::Event(event) => {
                let (start, end) = event.span(zone)?;
                (start < window.to && end > window.from).then(|| Scheduled { busy: event.holds_time(), event, start, end })
            }
            Object::Task(_) => None,
        })
        .collect();
    found.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.event.base.oid.cmp(&b.event.base.oid)));
    Ok(found)
}
```

Add `pub(crate) mod agenda;` to `use_cases/mod.rs`, the form every
module there takes, and the export to `lib.rs`. `dam-application` already depends on `jiff`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p dam-application && just gates`
Expected: every test passes; gates clean.

- [ ] **Step 5: Commit**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-application
SKIP_AI_COMMIT=1 git commit -m "feat(agenda): events overlapping a window, each with whether it holds time"
```

---

### Task 15: The `dam agenda` verb and its document

PR 6.

The document is a contract with programs that read busy times, so its rules are stated once here and
pinned by a test the executor must not loosen:

- One top-level key, `events`, an array ordered by start and then oid.
- Every event the window overlaps is a row, busy or not, cancelled ones included.
- `start` and `end` are integers, epoch seconds, `end` exclusive. `busy` is a boolean: whether the
  event holds the calendar owner's time (Task 13's rule).
- The other keys are dam's own vocabulary: `oid` (full), `subject`, `path`, `labels`, `status`,
  `transparency`, `all_day`.
- Keys are only ever added. None is renamed, removed, or changes type or meaning. A reader ignores a
  key it does not know.

With no flags the window opens now and closes 24 hours later, so a program that runs
`dam agenda --json` with no other argument gets every event in progress and every event of the next
day. A day of events measures a few kilobytes.

**Files:**
- Modify: `crates/dam-cli/src/args/reading.rs` (`AgendaArgs`), `crates/dam-cli/src/args.rs`
  (`Command::Agenda`, re-export)
- Create: `crates/dam-cli/src/commands/agenda.rs`
- Modify: `crates/dam-cli/src/commands/mod.rs` (`mod agenda;`, dispatch, `pulls_a_stale_remote`)
- Create: `crates/dam-cli/tests/agenda.rs`
- Modify: `crates/dam-cli/tests/support/sandbox.rs` (`TZ=UTC` on every run)
- Modify: `README.md`

**Interfaces:**
- Consumes: `agenda`, `Window`, `Scheduled` (Task 14); `when_flag` (`commands/parsing.rs`);
  `maybe_pull_stale`; `object_json`.
- Produces:

```rust
// args/reading.rs
#[derive(Args, Debug)]
pub(crate) struct AgendaArgs {
    /// A query or saved filter that narrows the events, read the way `ls` reads one.
    pub(crate) query: Option<String>,
    /// Where the window opens: a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM. Default: now.
    #[arg(long)]
    pub(crate) from: Option<String>,
    /// Where the window closes, exclusive. Default: 24 hours after it opens.
    #[arg(long)]
    pub(crate) to: Option<String>,
}

// commands/agenda.rs
pub(crate) fn run_agenda(ctx: &mut Context, args: AgendaArgs) -> Result<Report, CliError>;
```

- [ ] **Step 1: Write the failing unit tests**

```rust
// crates/dam-cli/src/commands/agenda.rs, at the bottom
#[cfg(test)]
mod tests {
    use dam_domain::{Attendee, Event, Object, Oid, ResponseStatus, When};

    use super::*;
    use crate::testing::context;

    fn standup() -> Event {
        let start = "2026-09-18T14:00:00+00:00[UTC]".parse().unwrap();
        let end = "2026-09-18T14:30:00+00:00[UTC]".parse().unwrap();
        let mut e = Event::new(Oid::generate(&mut |b: &mut [u8]| b.fill(0x3f)), "standup", When::At(start), When::At(end));
        e.base.labels.insert("work".into());
        e
    }

    fn args(from: Option<&str>, to: Option<&str>) -> AgendaArgs {
        AgendaArgs { query: None, from: from.map(str::to_string), to: to.map(str::to_string) }
    }

    /// The contract, whole: a change to any key here is a change to what
    /// programs reading busy times rely on.
    #[test]
    fn the_agenda_document_keeps_its_contract() {
        let mut ctx = context(); // now is 2026-09-18T00:00:00Z
        ctx.store.put(&Object::Event(standup())).unwrap();
        let report = run_agenda(&mut ctx, args(None, None)).unwrap();
        assert_eq!(
            report.data,
            serde_json::json!({"events": [{
                "oid": "3f".repeat(20), "subject": "standup", "path": "", "labels": ["work"],
                "status": "confirmed", "transparency": "busy", "all_day": false,
                "start": 1_789_740_000, "end": 1_789_741_800, "busy": true
            }]})
        );
    }

    #[test]
    fn with_no_flags_the_window_is_now_to_a_day_later() {
        let mut ctx = context();
        ctx.store.put(&Object::Event(standup())).unwrap();
        assert_eq!(run_agenda(&mut ctx, args(None, None)).unwrap().data["events"].as_array().unwrap().len(), 1);
        let later = run_agenda(&mut ctx, args(Some("2026-09-19"), None)).unwrap();
        assert!(later.data["events"].as_array().unwrap().is_empty());
    }

    #[test]
    fn a_declined_meeting_is_listed_as_not_busy() {
        let mut ctx = context();
        let mut declined = standup();
        declined.attendees = vec![Attendee { email: "me@x".into(), response: ResponseStatus::Declined, is_self: true }];
        ctx.store.put(&Object::Event(declined)).unwrap();
        assert_eq!(run_agenda(&mut ctx, args(None, None)).unwrap().data["events"][0]["busy"], false);
    }

    #[test]
    fn a_window_that_closes_before_it_opens_is_a_usage_error() {
        let mut ctx = context();
        let err = run_agenda(&mut ctx, args(Some("2026-09-20"), Some("2026-09-19"))).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(err.to_string().contains("--to"), "{err}");
    }

    #[test]
    fn the_human_line_says_busy_or_free_with_local_times() {
        let mut ctx = context();
        ctx.store.put(&Object::Event(standup())).unwrap();
        let report = run_agenda(&mut ctx, args(None, None)).unwrap();
        assert_eq!(report.human, "3f3f3f3  busy  2026-09-18 14:00  2026-09-18 14:30  standup");
    }
}
```

`dam agenda --no-pull` being accepted is pinned through the real binary in Step 2
(`dam_agenda_takes_no_pull_and_answers_from_the_store`); every other verb's refusal of the flag is
already pinned by the existing `--no-pull` tests and must stay green.

- [ ] **Step 2: Write the failing binary test**

In `tests/support/sandbox.rs`, `command` adds `.env("TZ", "UTC")`, so a date on the command line
means the same instant on every machine.

```rust
// crates/dam-cli/tests/agenda.rs
mod support;

use support::sandbox::Sandbox;

/// What a program reading busy times relies on, read strictly: the one key,
/// integer seconds, a boolean, and nothing else required.
fn busy_intervals(document: &str) -> Vec<(u64, u64, bool)> {
    let value: serde_json::Value = serde_json::from_str(document).unwrap();
    value["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["start"].as_u64().unwrap(), e["end"].as_u64().unwrap(), e["busy"].as_bool().unwrap()))
        .collect()
}

#[test]
fn dam_agenda_json_answers_every_event_in_the_window_as_intervals() {
    let _guard = support::guard("dam_agenda_json_answers_every_event_in_the_window_as_intervals");
    let sb = Sandbox::new();
    sb.new_object(&["--event", "standup", "--start", "2026-09-25T14:00", "--end", "2026-09-25T14:30"]);
    sb.new_object(&["--event", "offsite", "--start", "2026-09-25", "--end", "2026-09-26"]);
    sb.new_object(&["--event", "next week", "--start", "2026-10-02T09:00", "--end", "2026-10-02T10:00"]);
    let (ok, out, err) = sb.dam(&["agenda", "--from", "2026-09-25", "--to", "2026-09-26", "--json"]);
    assert!(ok, "{err}");
    assert_eq!(busy_intervals(&out), vec![(1_790_294_400, 1_790_380_800, true), (1_790_344_800, 1_790_346_600, true)]);
}

#[test]
fn dam_agenda_takes_no_pull_and_answers_from_the_store() {
    let _guard = support::guard("dam_agenda_takes_no_pull_and_answers_from_the_store");
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&["agenda", "--no-pull", "--json"]);
    assert!(ok, "{err}");
    assert_eq!(busy_intervals(&out), vec![]);
}
```

The sandbox's `dam` runs already set `HOME`, `XDG_CONFIG_HOME` and `XDG_DATA_HOME` to the temporary
directory; keep it that way.

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p damnit agenda`
Expected: unresolved `AgendaArgs` and `run_agenda`; the binary tests fail on `unrecognized
subcommand 'agenda'`.

- [ ] **Step 4: Implement**

```rust
// crates/dam-cli/src/commands/agenda.rs
//! `dam agenda`: events as intervals over a window, each saying whether it
//! holds time.

use dam_application::{Scheduled, Window, agenda};
use dam_domain::{Object, Timestamp};
use jiff::SignedDuration;

use crate::args::AgendaArgs;
use crate::commands::parsing::when_flag;
use crate::commands::remote::maybe_pull_stale;
use crate::context::Context;
use crate::error::CliError;
use crate::output::{Report, object_json};

/// How long the window stays open when `--to` is not given.
const DEFAULT_SPAN: SignedDuration = SignedDuration::from_hours(24);

pub(crate) fn run_agenda(ctx: &mut Context, args: AgendaArgs) -> Result<Report, CliError> {
    let window = window(ctx, &args)?;
    maybe_pull_stale(ctx)?;
    let found = agenda(ctx.store.as_ref(), ctx.clock.as_ref(), &ctx.config, args.query.as_deref(), window, &ctx.tz)?;
    let rows = found.iter().map(row).collect::<Result<Vec<_>, _>>()?;
    Ok(Report {
        human: found.iter().map(|s| line(s, &ctx.tz)).collect::<Vec<_>>().join("\n"),
        data: serde_json::json!({ "events": rows }),
    })
}

fn window(ctx: &Context, args: &AgendaArgs) -> Result<Window, CliError> {
    let instant = |flag: &str, text: &str| -> Result<Timestamp, CliError> {
        when_flag(ctx, flag, text)?
            .instant(&ctx.tz)
            .ok_or_else(|| CliError::Usage(format!("--{flag}: {text} is outside the calendar")))
    };
    let from = match &args.from {
        Some(text) => instant("from", text)?,
        None => ctx.clock.now(),
    };
    let to = match &args.to {
        Some(text) => instant("to", text)?,
        None => from.saturating_add(DEFAULT_SPAN).unwrap_or(from),
    };
    if to <= from {
        return Err(CliError::Usage("--to must be later than --from".into()));
    }
    Ok(Window { from, to })
}

/// Dam's own summary of the event beside the three keys a scheduler reads.
fn row(s: &Scheduled) -> Result<serde_json::Value, CliError> {
    let wire = object_json(&Object::Event(s.event.clone()))?;
    Ok(serde_json::json!({
        "oid": wire["oid"], "subject": wire["subject"], "path": wire["path"], "labels": wire["labels"],
        "status": wire["event"]["status"], "transparency": wire["event"]["transparency"],
        "all_day": s.event.start.is_all_day(),
        "start": s.start.as_second(), "end": s.end.as_second(), "busy": s.busy,
    }))
}

fn line(s: &Scheduled, zone: &jiff::tz::TimeZone) -> String {
    let local = |t: Timestamp| {
        let z = t.to_zoned(zone.clone());
        if s.event.start.is_all_day() { z.strftime("%Y-%m-%d").to_string() } else { z.strftime("%Y-%m-%d %H:%M").to_string() }
    };
    let mut cols = vec![
        s.event.base.oid.short().to_string(),
        if s.busy { "busy" } else { "free" }.to_string(),
        local(s.start),
        local(s.end),
        s.event.base.subject.clone(),
    ];
    if !s.event.base.path.as_str().is_empty() {
        cols.push(s.event.base.path.as_str().to_string());
    }
    cols.join("  ")
}
```

The window is read before the stale pull so a mistyped flag exits 2 without spawning a helper. In
`args.rs` add `/// Events over a window, each with whether it holds time.` and `Agenda(AgendaArgs)`
after `Ls`, and re-export `AgendaArgs`. In `commands/mod.rs` add `mod agenda;`,
`Command::Agenda(a) => agenda::run_agenda(ctx, a),` and extend `pulls_a_stale_remote` to
`Command::Ls(_) | Command::Show(_) | Command::Agenda(_)`; the `--no-pull` refusal text in `main.rs`
becomes "--no-pull applies to ls, show and agenda, the reads that pull a stale remote".
`dam-cli` already depends on `jiff`, so `SignedDuration` needs no new dependency.

README, under "Reading it from a program":

```markdown
`dam agenda` lists events as intervals, for a program that schedules around busy time:

    {"events": [{"oid": "98d878...", "subject": "standup", "path": "me@example.com/",
     "labels": [], "status": "confirmed", "transparency": "busy", "all_day": false,
     "start": 1790344800, "end": 1790346600, "busy": true}]}

`start` and `end` are epoch seconds, `end` exclusive; `busy` says whether the event holds your time:
not cancelled, shown busy, and not declined by you. Every event the window overlaps is listed, busy
or not. With no flags the window runs from now to 24 hours later; `--from` and `--to` take the same
words `--due` does, and a query narrows it the way `dam ls` reads one. Keys are only ever added, never
renamed or retyped. Like `ls`, `agenda` pulls a stale remote first unless given `--no-pull`.
```

- [ ] **Step 5: Run to verify pass**

Run: `just gates`
Expected: clean; the two binary tests inside a second each.

- [ ] **Step 6: Commit, and apply the amendment once approved**

```bash
SKIP_AI_COMMIT=1 git add crates/dam-cli README.md
SKIP_AI_COMMIT=1 git commit -m "feat(agenda): dam agenda lists events as intervals with a busy reading"
```

Once Amendment A5 is approved, apply it in its own commit, `docs(spec): dam agenda and its
document`. Pending: stop and ask.

---

## Proposed spec amendments

**None of these is applied to the spec by this plan.** Each is the text the named task would write
into `docs/superpowers/specs/2026-09-18-damnit-design.md` once the operator approves it. Each carries
a recommendation.

### A1. A helper is given its remote's name and address (Task 4)

**Where:** "Remotes and helpers", the table and the paragraph after it.

**Why the spec needs it:** the spec's config example has `url = "gcal::"` and says the helper reads
`DAM_<REMOTE>_<NAME>`, but a helper is never told which remote it serves, so it can only guess its
own variable names (the Todoist helper hardcodes `DAM_TODOIST_API_TOKEN`), and there is nowhere for
the operator to say which calendars to read.

**Proposed text,** a new table row and paragraph:

> | `hg::<address>` gives `git-remote-hg` the name and the address | `gcal::<address>` runs `dam-remote-gcal <remote> <address>` |
>
> `dam` runs a helper as `dam-remote-<helper> <remote> <address>`, git's own invocation: the
> remote's name, then the text after `::` in its url, empty when there is none. Both are always
> passed and neither is ever a secret. A helper reads its credentials under that remote name, and
> the address is the helper's to read.

**Recommendation: approve.** It is git's exact shape, costs one line in the process adapter, and the
Todoist helper keeps working unchanged because it ignores its arguments.

### A2. An attendee says whether it is the calendar's own (Task 5)

**Where:** "Data model", the Event table's `attendees` row.

**Proposed text:**

> | `attendees` | list | Email plus response: accepted, declined, tentative, needs action; and `self`, whether this attendee is the calendar the event was read from. |

**Why:** without it a meeting the operator declined still reads as busy. Google's event resource
carries `attendees[].self`, so the spec's rule that every event field maps to one on Google's
resource still holds.

**Recommendation: approve.**

### A3. A pull reports a cancellation by id (Task 6)

**Where:** "Protocol", after the pull request and response, and "Sync rules", rule 3.

**Proposed text:**

> A pull response may carry `cancelled`, a list of remote ids the remote cancelled without restating
> them. `dam` moves the event it tracks under each id to `cancelled` through the same rules as any
> pulled change, keeps tracking it, and reports attached tasks as rule 4 says. An id it does not
> track, or one naming a task, changes nothing.

Rule 3 gains: "A cancellation upstream is a status, not a removal: the event stays, cancelled."

**Why:** Google reports a deleted event as a cancelled item that may carry only its id. Reported as
`removed`, it would stay confirmed in dam and keep reading as busy.

**Recommendation: approve.**

### A4. dam-remote-gcal, read-only, and how its refresh token is minted (Tasks 3 and 12)

**Where:** "Helpers in this repository", "Credentials", and the `[remote.gcal]` config example.

**Proposed text,** under "Helpers in this repository":

> `dam-remote-gcal`, version one, is read-only. Its address names the calendars it reads,
> comma-separated, and an empty address reads the primary calendar: `gcal::primary,team@group.calendar.google.com`.
> A pull reads the week behind and the ninety days ahead, with recurring events as their instances,
> so it declares no `recurrence`, and none of `labels`, `depends` or `reminders`, which stay `dam`'s
> own. An event Google cancelled, and one that left that window early, arrive by id as cancelled.
> A push is answered per mutation with a refusal naming the helper as read-only, and nothing reaches
> Google. It reads one variable of its own, `DAM_GCAL_BASE_URL`, under the same loopback-only rule
> as the Todoist helper's.
>
> **Signing in.** The package also installs `dam-gcal-sign-in`, which the operator runs once:
> `dam-gcal-sign-in --client-id <id>`, the client secret piped on standard input and refused from a
> terminal or an argument. It walks Google's installed-app consent (a loopback redirect, PKCE with
> S256), asks for `calendar.events.readonly` alone, prints the refresh token once on standard output
> and writes it nowhere. The operator stores it where `refresh_token_command` reads it.

The config example's `[remote.gcal]` gains the comment `# url = "gcal::primary,team@group.calendar.google.com" reads two calendars`.

**Why:** the spec reads the refresh token from the vault but never says how a person gets one, and
it says nothing about which calendars, which slice of time, or what read-only means for a push.

**Recommendation: approve.** Decisions inside it the operator may want to change: the window (a week
back, ninety days ahead), the scope (read-only, so two-way sync later means signing in again), and
the sign-in being a binary of its own rather than a `dam` verb.

### A5. `dam agenda` (Task 15)

**Where:** "Commands", "Reading", and the `--no-pull` paragraph.

**Proposed text:**

> ```
> dam agenda [<query>] [--from <when>] [--to <when>] [--json|--toon] [--no-pull]
> ```
>
> `dam agenda` lists the events a window overlaps as intervals, for a program that schedules around
> busy time. It answers `{"events": [...]}`, ordered by start: each row carries `oid`, `subject`,
> `path`, `labels`, `status`, `transparency` and `all_day`, and three keys a scheduler reads,
> `start` and `end` as epoch seconds with `end` exclusive, and `busy`, whether the event holds the
> calendar owner's time: not cancelled, shown busy, and not declined by the calendar's own attendee.
> Every event the window overlaps is listed, busy or not. An all-day event runs midnight to midnight
> where `dam` runs. With no flags the window opens now and closes 24 hours later. Keys are only ever
> added; none is renamed, removed or retyped.

The `--no-pull` paragraph's "`ls` and `show`" becomes "`ls`, `show` and `agenda`".

**Why:** `dam ls --json` carries events with their times as zoned text nested under `event`, which a
scheduler would have to parse and filter itself, and nothing in dam says whether an event holds time.

**Recommendation: approve,** including the name. Alternatives considered for the name were weaker
because no git word fits and `agenda` is the word calendar tools already use for this listing.

## Follow-ups this plan does not do

- **Two-way sync with Google Calendar.** Its own spec and plan; it needs the wider
  `calendar.events` scope, so a person signs in again.
- **The Todoist helper reads its remote's name from its arguments** once A1 lands, instead of
  hardcoding `DAM_TODOIST_API_TOKEN`. A one-task change to that helper.
- **Commits no remote will take still count as owed.** A gcal pull commit is marked pushed for gcal
  only, so `dam status` lists it as owed to Todoist until the next `dam push` skips it; the reverse
  holds for Todoist's pull commits. This predates this plan and first shows with a second remote.
- **`fromGmail` events** read as `default`, because dam's `event_type` has no value for them.
- **A notice per refused push.** dam resends a refused mutation on every push and records a notice
  each time; notices are capped, not deduplicated.

## Self-review notes

- Spec coverage: the helper (Tasks 7 to 12) pulls with an OAuth refresh token, declares the spec's
  three credentials, reads them through dam's existing `*_command` resolution, maps every Event field
  the spec lists, and ships as its own package. The sign-in (Tasks 1 to 3) answers the spec's silence
  on minting the token. The listing (Tasks 13 to 15) is generic, names no consumer, and its default
  window with argv alone covers every event in progress now.
- Read-only is enforced twice: the scope Google grants, and a push path that reads no credential and
  sends no request (`a_push_is_refused_per_mutation_without_a_credential_or_a_request`).
- Secrets: every test that could leak one asserts its absence from standard error, the error string
  and `Debug`. The helper spawns no process, so no credential reaches a child's environment; the
  sign-in reads the client secret from standard input, never argv or the environment.
- Type consistency: `Secret`, `Client`, `Endpoints`, `ApiError`, `Window` (helper) and `Window`
  (application) are distinct types in distinct crates and are never imported into one file together.
- The code in this plan was compiled and run before it was written down, in scratch copies outside
  this repository, on 2026-09-22 at `main` `bae8a19`: Tasks 1 to 3 and 7 to 12 as a standalone
  `dam-remote-gcal` over a copy of `dam-protocol` carrying Tasks 4 to 6, and Tasks 4, 5, 6, 13, 14
  and 15 applied to a clone of this repository. `cargo clippy --all-targets -- -D warnings` was
  clean for both, the helper's 63 tests and the workspace's 547 passed, and `just size` passed.
  Prose-described pieces (`checked_base`, `Secret`, `bounded`, the resources) were written from the
  task text alone, which is what an executor will have.
