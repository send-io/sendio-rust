# sendio

[![Crate Version](https://img.shields.io/crates/v/sendio)](https://crates.io/crates/sendio)
[![Crate Docs](https://img.shields.io/docsrs/sendio)](https://docs.rs/sendio)

The official Rust SDK for the [send.io](https://send.io) email API.

- Crate: `sendio`
- API reference: https://developer.send.io/docs/api

## Installation

```sh
cargo add sendio
cargo add tokio -F macros,rt-multi-thread
```

The client is async by default. Enable the `blocking` feature for a synchronous client (the two are mutually exclusive). The crate uses [reqwest](https://github.com/seanmonstar/reqwest) and [serde](https://github.com/serde-rs/serde) internally.

## Setup

Create an API key in the send.io dashboard. Keys start with `mx_`. Keep it out of source code and pass it through the environment:

```sh
export SENDIO_API_KEY="mx_..."
```

## Quick start: send an email

```rust,no_run
use sendio::types::CreateEmailBaseOptions;
use sendio::{Result, Sendio};

#[tokio::main]
async fn main() -> Result<()> {
    // Reads SENDIO_API_KEY (and SENDIO_BASE_URL, if set).
    let sendio = Sendio::default();

    let email = CreateEmailBaseOptions::new(
        "Acme <onboarding@send.io>",
        ["delivered@send.io"],
        "Hello from send.io",
    )
    .with_html("<p>It works!</p>");

    let sent = sendio.emails.send(email).await?;
    println!("id: {}", sent.id);
    Ok(())
}
```

This example sends from `onboarding@send.io` to the test address `delivered@send.io`, so it works on a new account before you add a domain. `onboarding@send.io` can only send to test addresses. To email real people, send from an address on a domain you verified. A free mailbox works too once Mailboxes shows it can send.

You can also pass the key explicitly with `Sendio::new("mx_...")`.

Idempotency keys are supported on sends:

```rust,no_run
# use sendio::{Sendio, Result, types::CreateEmailBaseOptions};
# async fn run(sendio: Sendio, email: CreateEmailBaseOptions) -> Result<()> {
let sent = sendio
    .emails
    .send(email.with_idempotency_key("welcome-user-123"))
    .await?;
# Ok(())
# }
```

## Batch sending

Send up to 100 emails in one request:

```rust,no_run
# use sendio::{Sendio, Result, types::CreateEmailBaseOptions};
# async fn run(sendio: Sendio) -> Result<()> {
let emails = vec![
    CreateEmailBaseOptions::new("Acme <onboarding@yourdomain.com>", ["delivered@send.io"], "First")
        .with_html("<p>one</p>"),
    CreateEmailBaseOptions::new("Acme <onboarding@yourdomain.com>", ["delivered@send.io"], "Second")
        .with_html("<p>two</p>"),
];

let sent = sendio.batch.send(emails).await?;
for email in sent {
    println!("{}", email.id);
}
# Ok(())
# }
```

## Retrieve an email

```rust,no_run
# use sendio::{Sendio, Result};
# async fn run(sendio: Sendio) -> Result<()> {
let email = sendio.emails.get("4ef9a417-02e9-4d39-ad75-9611e0fcc33c").await?;
println!("{} {:?}", email.subject, email.last_event);
# Ok(())
# }
```

## Transactional emails

A transactional email (a sign-in code, a password reset, a receipt) is a saved send of a published template: it
keeps the From, subject, reply-to and tracking (off by default), and its content is always the template's
published version. Create it once, then send it by id with only the recipients and the variables.

```rust,no_run
# use sendio::{Sendio, Result};
# use sendio::types::{CreateEmailBaseOptions, CreateTransactionalEmailOptions};
# async fn run(sendio: Sendio) -> Result<()> {
let tx = sendio
    .transactional
    .create(CreateTransactionalEmailOptions::new(
        "Sign-in code",
        "signin-code", // a published template, by id or alias
        "Acme <security@yourdomain.com>",
        "Your code is {{CODE}}",
    ))
    .await?;

let email = CreateEmailBaseOptions::transactional(&tx.id, ["ada@example.com"])
    .with_variable("CODE", "123456");
let _sent = sendio.emails.send(email).await?;
# Ok(())
# }
```

`sendio.transactional` also has `list` (`ListTransactionalEmailsOptions::with_search`, `with_template_id`), `get`,
`update` (`with_expected_updated_at` gets a 409 instead of overwriting someone else's change), `delete`,
`send_test` and `stats`. A republish of the template, or an update, reaches every send within 10 seconds.

Broadcasts can be made from a published template too: `with_template_id` (and `with_template_version_id`) on
`CreateBroadcastOptions`, and `UpdateBroadcastOptions::detach_template` to keep the content but drop the link.
`Broadcast::unfillable_variables` lists the template variables a broadcast cannot fill; while it is not empty,
sending is refused with 422.

## Domains

```rust,no_run
# use sendio::{Sendio, Result};
# async fn run(sendio: Sendio) -> Result<()> {
use sendio::list_opts::ListOptions;
use sendio::types::CreateDomainOptions;

let domain = sendio
    .domains
    .create(CreateDomainOptions::new("yourdomain.com"))
    .await?;
// Add domain.records to your DNS, then:
let _ = sendio.domains.verify(&domain.id).await?;

let list = sendio.domains.list(ListOptions::default()).await?;
for d in list.data {
    println!("{} {:?}", d.name, d.status);
}
# Ok(())
# }
```

To receive email on a domain, add it with receiving enabled (the records then include an MX record). Turn receiving off or back on later with `update`. Off stops send.io accepting email for the domain; remove its MX record afterwards. Turning it back on asks for the domain's MX record; publish it to start receiving email. Sending can't be disabled.

```rust,no_run
# use sendio::{Sendio, Result};
# async fn run(sendio: Sendio) -> Result<()> {
use sendio::types::{
    CreateDomainOptions, DomainCapabilitiesOptions, DomainCapabilityStatus, DomainChanges,
};

let domain = sendio
    .domains
    .create(
        CreateDomainOptions::new("yourdomain.com").with_capabilities(
            DomainCapabilitiesOptions::new().with_receiving(DomainCapabilityStatus::Enabled),
        ),
    )
    .await?;

let _ = sendio
    .domains
    .update(
        &domain.id,
        DomainChanges::new().with_capabilities(
            DomainCapabilitiesOptions::new().with_receiving(DomainCapabilityStatus::Disabled),
        ),
    )
    .await?;
# Ok(())
# }
```

## Mailboxes

Mailboxes are email accounts with an IMAP/SMTP login. The domain must be verified (the API answers 409 if the address exists).

```rust,no_run
# use sendio::{Sendio, Result};
# async fn run(sendio: Sendio) -> Result<()> {
use sendio::types::{CreateMailboxOptions, UpdateMailboxOptions};

let mailbox = sendio
    .mailboxes
    .create(CreateMailboxOptions::address("sales@yourdomain.com"))
    .await?;
// Returned once, when IMAP/SMTP is ready. The password is also an API key for this mailbox.
if let Some(creds) = &mailbox.credentials {
    println!("{} {} {}", creds.username, creds.imap.host, creds.smtp.host);
}

for m in sendio.mailboxes.list().await?.data {
    println!("{} {:?} {:?}", m.address, m.imap_smtp, m.health);
}

let _ = sendio
    .mailboxes
    .update(&mailbox.id, UpdateMailboxOptions::new(false))
    .await?;

// The old password stops working.
let reset = sendio.mailboxes.reset_password(&mailbox.id).await?;
println!("{}", reset.credentials.password);

// Also removes the agent inbox on it, if any.
let _ = sendio.mailboxes.delete(&mailbox.id).await?;
# Ok(())
# }
```

The client also covers API keys, segments, contacts (with topics, properties and imports), broadcasts, templates, topics, webhooks (create, list, update, delete, events and replays, signing secret rotation, signature verification), receiving (inbound emails), logs, automations (with automation templates), events, suppressions, OAuth grants and usage. See the crate docs and the `examples/` directory.

## Verify webhooks

Check the `svix-id`, `svix-timestamp` and `svix-signature` headers of an incoming webhook request against the raw body and your webhook's signing secret (`whsec_...`). The timestamp must be within 5 minutes (change it with `with_tolerance`), and the header may carry several `v1` signatures (for example during a secret rotation).

```rust,no_run
use sendio::types::{VerifyWebhookOptions, WebhookHeaders};

# fn handle(raw_body: &str, svix_id: &str, svix_timestamp: &str, svix_signature: &str) {
let headers = WebhookHeaders {
    id: svix_id,
    timestamp: svix_timestamp,
    signature: svix_signature,
};
let secret = std::env::var("SENDIO_WEBHOOK_SECRET").unwrap();

match sendio::verify_webhook(VerifyWebhookOptions::new(raw_body, headers, &secret)) {
    Ok(()) => println!("{:?}", sendio::events::try_parse_event(raw_body)),
    Err(err) => eprintln!("rejected: {err}"), // respond 400
}
# }
```

`examples/axum-verify-event-middleware.rs` does the same as Axum middleware.

## Configuration

| Setting | How |
|---|---|
| API key | `Sendio::new(key)`, or `SENDIO_API_KEY` with `Sendio::default()` |
| Base URL | `SENDIO_BASE_URL` environment variable (default `https://api.send.io`) |
| Base URL per client | `ConfigBuilder::base_url(...)`, which takes precedence over the environment |
| HTTP client / timeouts | `ConfigBuilder::client(reqwest::Client)` or `Sendio::with_client(...)` |
| Client-side rate limit | `SENDIO_RATE_LIMIT` (requests per second, default 9, async client only) |

```rust,no_run
use sendio::{Config, Sendio};
use std::time::Duration;

let http = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()
    .unwrap();

let sendio = Sendio::with_config(
    Config::builder("mx_...")
        // e.g. a local mock server in tests
        .base_url("http://localhost:8080".parse().unwrap())
        .client(http)
        .build(),
);
```

The base URL is accepted with or without a trailing slash, and a path prefix (for example `http://localhost:8080/v1`) is kept. Every request sends `User-Agent: sendio-rust:1.2.0`.

### Features

- `native-tls`: system-native TLS. Enabled by default.
- `rustls-tls`: TLS backed by `rustls`.
- `blocking`: synchronous client.
- WASM is supported (see `examples/cf-worker`).

## Error handling

All methods return `sendio::Result<T>`. API errors come back as `Error::Sendio(ErrorResponse)` with the HTTP status, error name and message; `ErrorResponse::kind()` maps the name to an `ErrorKind`. Rate limits are reported as `Error::RateLimit`, and the `rate_limit` module has helpers to retry automatically.

```rust,no_run
# use sendio::{Sendio, types::CreateEmailBaseOptions};
# async fn run(sendio: Sendio, email: CreateEmailBaseOptions) {
match sendio.emails.send(email).await {
    Ok(sent) => println!("sent {}", sent.id),
    Err(sendio::Error::Sendio(err)) => {
        eprintln!("send.io rejected the request: {} ({:?})", err.message, err.kind());
    }
    Err(sendio::Error::RateLimit { ratelimit_reset, .. }) => {
        eprintln!("rate limited, reset in {ratelimit_reset:?}s");
    }
    Err(other) => eprintln!("request failed: {other}"),
}
# }
```

## Testing

```sh
cargo test
```

Tests that call a live API are compiled in but ignored by default. To run them against a test account:

```sh
SENDIO_API_KEY=mx_... SENDIO_BASE_URL=https://api.send.io cargo test -- --ignored
```

A short live smoke test (sends one real email) is in `examples/smoke.rs`:

```sh
SENDIO_API_KEY=mx_... cargo run --example smoke
```

## Documentation

Full API reference: https://developer.send.io/docs/api

## Releasing

For maintainers. Releases go out from GitHub Actions on the public copy of this SDK, [send-io/sendio-rust](https://github.com/send-io/sendio-rust), not from a laptop:

1. In send-io-core, bump the version in `Cargo.toml`, add the `CHANGELOG.md` entry and update the version shown in the console docs (a console test compares them), then merge into `develop`.
2. Sync the public copy: replace its files with this folder at the merged commit (`git archive <sha>:sdks/rust`), commit it as `sync with send-io-core <short sha>` and push `main`.
3. Optional: run the Publish workflow by hand. It only checks the crates.io token and publishes nothing.
4. Push the tag `vX.Y.Z` on the public repository. The Publish workflow (`.github/workflows/publish.yml`) checks that the tag matches the version, runs the tests and publishes to crates.io.

## License

MIT. See [LICENSE.txt](./LICENSE.txt) and [NOTICE](./NOTICE).
