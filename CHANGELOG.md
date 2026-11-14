# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, a minor version
may contain breaking changes.

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
