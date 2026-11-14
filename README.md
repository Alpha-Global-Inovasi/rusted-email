# rusted-email

Email library for Rust services, with SMTP and mock providers and Handlebars templates. Part of the Rusted ecosystem, built by PT Alpha Global Inovasi.

This crate was previously named `cf-email` and distributed from Git only.

## Installation

```toml
[dependencies]
rusted-email = "0.2"
```

### TLS backend

SMTP connections use rustls with aws-lc-rs by default, and certificates are verified against the operating system's trust store. To use the platform TLS library (OpenSSL on Linux) instead:

```toml
rusted-email = { version = "0.2", default-features = false, features = ["native-tls"] }
```

## Environment Variables

| Variable             | Required           | Description                                       |
| -------------------- | ------------------ | ------------------------------------------------- |
| `EMAIL_DRIVER`       | No                 | `mock` for the mock provider, otherwise SMTP      |
| `EMAIL_FROM_ADDRESS` | Yes                | Sender address                                    |
| `EMAIL_FROM_NAME`    | Yes                | Sender name                                       |
| `SMTP_HOST`          | When using SMTP    | SMTP host                                         |
| `SMTP_PORT`          | When using SMTP    | SMTP port                                         |
| `SMTP_USERNAME`      | No                 | SMTP username                                     |
| `SMTP_PASSWORD`      | No                 | SMTP password                                     |
| `SMTP_USE_TLS`       | No                 | `true` (default) for TLS: implicit TLS on port 465, STARTTLS on any other port. Otherwise plaintext |

## Usage

```rust,ignore
use rusted_email::{EmailClient, EmailConfig, EmailRequest};
use serde_json::json;

let client = EmailClient::new(EmailConfig::from_env()?).await?;
client.register_template("welcome", "<p>Hi {{name}}</p>").await?;

let request = EmailRequest::builder()
    .to("user@example.com")
    .subject("Welcome")
    .template("welcome", json!({ "name": "Kanda" }))
    .build()?;

client.send(request).await?;
```

## Development

```bash
cargo test
cargo clippy --all-targets
cargo deny check                        # licenses and advisories (needs cargo-deny)
```

Minimum supported Rust version: 1.89. Raising it is a minor-version change.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
