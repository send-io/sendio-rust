//! Live smoke test against a real send.io endpoint. Not part of the unit tests.
//!
//! ```sh
//! SENDIO_API_KEY=mx_... SENDIO_BASE_URL=https://api.send.io cargo run --example smoke
//! ```
//!
//! Optional: `SMOKE_FROM` (default `ada@compat-sdk.sendio-test.dev`) and `SMOKE_TO`
//! (default `delivered@send.io`).

#[cfg(feature = "blocking")]
fn main() {
    panic!("This example is meant for async mode.");
}

#[cfg(not(feature = "blocking"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use sendio::list_opts::ListOptions;
    use sendio::types::CreateEmailBaseOptions;
    use sendio::{DEFAULT_BASE_URL, Sendio};

    let env = |key: &str, default: &str| {
        std::env::var(key)
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| default.to_owned())
    };

    // Reads SENDIO_API_KEY and SENDIO_BASE_URL.
    let sendio = Sendio::default();
    println!("base URL: {}", env("SENDIO_BASE_URL", DEFAULT_BASE_URL));

    let from = env("SMOKE_FROM", "ada@compat-sdk.sendio-test.dev");
    let to = env("SMOKE_TO", "delivered@send.io");
    let email = CreateEmailBaseOptions::new(from, [to], "sendio-rust smoke test")
        .with_html("<p>Hello from the sendio Rust SDK</p>")
        .with_text("Hello from the sendio Rust SDK");

    let sent = sendio.emails.send(email).await?;
    println!("send: OK id = {}", sent.id);

    let mut failed = false;

    match sendio.emails.get(&sent.id).await {
        Ok(e) => println!(
            "get: OK id={} from={} last_event={:?}",
            e.id, e.from, e.last_event
        ),
        Err(err) => {
            println!("get: FAIL {err}");
            failed = true;
        }
    }

    match sendio.domains.list(ListOptions::default()).await {
        Ok(list) => println!("domains.list: OK {} domain(s)", list.data.len()),
        Err(err) => {
            println!("domains.list: FAIL {err}");
            failed = true;
        }
    }

    match sendio.api_keys.list(ListOptions::default()).await {
        Ok(list) => println!("api_keys.list: OK {} key(s)", list.data.len()),
        Err(err) => {
            println!("api_keys.list: FAIL {err}");
            failed = true;
        }
    }

    if failed {
        anyhow::bail!("smoke test failed");
    }
    Ok(())
}
