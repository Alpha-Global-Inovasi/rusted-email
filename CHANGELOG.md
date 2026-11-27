# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, a minor version
may contain breaking changes.

## [0.3.0] - 2026-11-27

### Changed

- The `text/plain` part of an HTML email is now a readable version of the HTML instead of
  "Please view this email in a modern client". Links keep their address as
  `text (address)`.
- `EmailContent` has a new variant, so an exhaustive `match` on it needs an arm for
  `Alternative`.

### Security

- Requires `lettre` 0.11.24, which fixes RUSTSEC-2026-0560, RUSTSEC-2026-0561, and
  RUSTSEC-2026-0562 (STARTTLS response injection, SMTP command injection through bare
  CR or LF, and a panic on an empty `EHLO` reply line) and the other lettre advisories
  published with them.

### Added

- `EmailContent::Alternative { html, text }` and `EmailRequestBuilder::html_with_text` to
  send your own text part.
- A template registered as `<name>.txt` is rendered as the text part of template `<name>`.
- `text::html_to_text` and `TemplateEngine::has_template`.

## [0.2.0] - 2026-11-15

### Changed

- Renamed from `cf-email` to `rusted-email` to match the Rusted brand. The repository
  moved to `Alpha-Global-Inovasi/rusted-email`, and the crate is published on crates.io.
- License is now `MIT OR Apache-2.0`, copyright PT Alpha Global Inovasi.
- SMTP connections use rustls with aws-lc-rs by default, verifying certificates against
  the operating system's trust store. The previous OpenSSL-based backend is available
  with `default-features = false, features = ["native-tls"]`.
- Dependencies are on their latest stable releases: `handlebars` 6, `thiserror` 2,
  `lettre` 0.11.23.
- Minimum supported Rust version is 1.89.

### Added

- `Default` for `TemplateEngine`.

### Migrating from cf-email 0.1

```toml
[dependencies]
rusted-email = "0.2"
```

Then replace `cf_email` with `rusted_email` in `use` statements. The API is unchanged.

## [0.1.1]

### Added

- Implicit TLS on port 465.

## [0.1.0]

Initial release as `cf-email`.
