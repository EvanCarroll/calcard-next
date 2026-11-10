# AGENTS.md

Guidance for AI coding agents working on `calcard-next`, a Rust library for
parsing, generating and converting iCalendar/JSCalendar and vCard/JSContact.

## Priorities

- **Correctness and performance come first.** Never sacrifice either one for
  backwards compatibility or a smaller changeset. Make the change the
  specification requires, even when it breaks the public API, changes output
  or touches many files. Mark breaking changes as described below.
- Don't keep known-wrong behaviour as a fallback, and don't work around a bug
  in one place while leaving it in another. Fix it where it lives. If that is
  out of scope, say so explicitly and file an issue rather than leaving it
  silently.
- Don't avoid restructuring code, or reformatting the lines you touch, to keep
  the diff small. Review a large diff carefully rather than shrinking it.

## Commits

- **Never add a `Co-Authored-By` trailer**, or any other AI attribution
  (trailers, footers, "Generated with" lines), to commits, tags, pull requests
  or release notes.
- Use [Conventional Commits](https://www.conventionalcommits.org/):
  `type(optional scope): summary`, with types such as `feat`, `fix`, `docs`,
  `refactor`, `perf`, `test`, `build`, `ci` and `chore`. Mark breaking changes
  with `!` (`feat!: ...`) or a `BREAKING CHANGE:` footer.
- Record user-visible changes in `CHANGELOG.md`.

## Building and testing

Run the tests in each feature configuration:

```sh
cargo test                                          # defaults: jmap, legacy-charsets
cargo test --no-default-features --features jmap    # without legacy-charsets
cargo test --all-features
cargo clippy --all-features --all-targets
```

- `tests/golden.rs` compares library output for the corpus in
  `tests/resources` with the files in `tests/golden`. After an intended
  output change, regenerate with `UPDATE_GOLDEN=1 cargo test --all-features --test golden`
  (and once more without `legacy-charsets` for `charsets_default.txt`), then
  review the diff before committing. On a mismatch the actual output is written
  next to the golden file as `*.actual` (git-ignored).
- The WebAssembly playground in `web/` is a separate crate and workspace:
  build it with `./web/build.sh`.

## Conventions

- Rust edition 2024. Use the `foo.rs` + `foo/` module layout; never create
  `mod.rs` files.
- Date-time types come from `jiff`. `Tz` IDs (`Tz::as_id`, `IanaTz` in
  `src/common/zones.rs`) are persisted by users: never renumber them; new zones
  may only be appended.
- Keep `jiff`'s time zone database bundled (`tzdb-bundle-always`) so results
  do not depend on the host system.
- Optional dependencies belong behind features; `legacy-charsets`
  (`encoding_rs`) is on by default and is disabled by size-sensitive builds
  such as the playground.
