//! Run this example with:
//!
//! ```sh
//! SENDIO_API_KEY=mx_your_api_key cargo run --example custom-config
//! ```

#[cfg(feature = "blocking")]
fn main() {
    panic!("This example is meant for async mode.");
}

#[cfg(not(feature = "blocking"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use sendio::types::CreateEmailBaseOptions;
    use sendio::{Config, Sendio};
    use std::time::Duration;

    const EMAIL_SEND_TIMEOUT: Duration = Duration::from_secs(10);

    let api_key = std::env::var("SENDIO_API_KEY").context(
        "SENDIO_API_KEY is expected in the environment for this example to run properly",
    )?;

    let sendio = Sendio::with_config(
        Config::builder(api_key)
            .base_url(
                // this is Sendio's default base url, but you can provide
                // your override here, which is especially helpful when running
                // numerous parallel tests and intercepting email requests
                // in each of them
                "https://api.send.io"
                    .parse()
                    .context("failed to parse URL")?,
            )
            .client(
                reqwest::Client::builder()
                    .timeout(EMAIL_SEND_TIMEOUT)
                    .build()
                    .context("failed to instrantiate an http client")?,
            )
            .build(),
    );

    let _resp = sendio
        .emails
        .send(
            CreateEmailBaseOptions::new("onboarding@yourdomain.com", ["delivered@send.io"], "Demo")
                .with_html("<strong>It works!</strong>")
                .with_text("It works!"),
        )
        .await?;

    Ok(())
}
