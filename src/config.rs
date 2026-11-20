#[cfg(not(feature = "blocking"))]
use governor::{
    Quota, RateLimiter,
    clock::MonotonicClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
};
#[cfg(feature = "blocking")]
use reqwest::blocking::{Client, RequestBuilder, Response};
#[cfg(not(feature = "blocking"))]
use reqwest::{Client, RequestBuilder, Response};
use reqwest::{Method, Url};
use reqwest::{StatusCode, header::USER_AGENT};
use std::{env, fmt};
#[cfg(not(feature = "blocking"))]
use std::{num::NonZeroU32, sync::Arc, time::Duration};

use crate::{Error, Result, error::types::ErrorResponse};

#[cfg(doc)]
use crate::Sendio;

/// Default send.io API base URL, used when neither [`ConfigBuilder::base_url`] nor the
/// `SENDIO_BASE_URL` environment variable is set.
pub const DEFAULT_BASE_URL: &str = "https://api.send.io";

/// Convenience builder for [`Config`].
///
/// This requires from you to set the API key ([`ConfigBuilder::new`]), but also
/// makes it possible to set a `reqwest` http client with your custom configuration
/// (see also [`Sendio::with_client`]) as well as an override for the Sendio's
/// base url to send requests to.
///
/// ```no_run
/// # use sendio::ConfigBuilder;
/// let http_client = reqwest::Client::builder()
///      .timeout(std::time::Duration::from_secs(10))
///      .build()
///      .unwrap();
///
/// // Make sure to not store secrets in code, instead consider using crates like `dotenvy`
/// // or `secrecy`.
/// let _config = ConfigBuilder::new("re_...")
///     // this can be your proxy's url (if any) or a test server url which
///     // is intercepting request and allows to inspect them later on
///     .base_url("http://wiremock:35353".parse().unwrap())
///     .client(http_client)
///     .build();
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigBuilder {
    api_key: String,
    base_url: Option<Url>,
    client: Option<Client>,
}

impl ConfigBuilder {
    /// Create new [`ConfigBuilder`] with `api_key` set.
    pub fn new<S>(api_key: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            api_key: api_key.into(),
            base_url: None,
            client: None,
        }
    }

    /// Set a custom Sendio's base url.
    ///
    /// This can be your proxy's url (if any) or a test server url which
    /// intercepting request and allows to inspect them later on.
    ///
    /// If not provided here, the `SENDIO_BASE_URL` environment variable will be
    /// used. If that is not not provided either - a [default] url will be used.
    ///
    /// [default]: https://developer.send.io/docs/api
    #[must_use]
    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Set custom http client.
    #[must_use]
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Builder's terminal method producing [`Config`].
    pub fn build(self) -> Config {
        Config::new(self.api_key, self.client.unwrap_or_default(), self.base_url)
    }
}

/// Configuration for `Sendio` client.
///
/// Use [`Config::builder`] to start constructing your custom configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    pub(crate) user_agent: String,
    pub(crate) api_key: String,
    pub(crate) base_url: Url,
    pub(crate) client: Client,
    #[cfg(not(feature = "blocking"))]
    limiter: Arc<
        RateLimiter<
            NotKeyed,
            InMemoryState,
            MonotonicClock,
            NoOpMiddleware<<MonotonicClock as governor::clock::Clock>::Instant>,
        >,
    >,
}

impl Config {
    /// Create new [`ConfigBuilder`] with `api_key` set.
    ///
    /// A convenience method, that, internally, will call [`ConfigBuilder::new`].
    pub fn builder<S>(api_key: S) -> ConfigBuilder
    where
        S: Into<String>,
    {
        ConfigBuilder::new(api_key.into())
    }

    /// Creates a new [`Config`].
    ///
    /// Note: the `base_url` parameter takes presedence over the `SENDIO_BASE_URL` environment
    /// variable.
    #[must_use]
    pub(crate) fn new(api_key: String, client: Client, base_url: Option<Url>) -> Self {
        let env_base_url = base_url.unwrap_or_else(|| {
            match env::var("SENDIO_BASE_URL") {
                Ok(env_var) if !env_var.trim().is_empty() => Url::parse(env_var.trim()),
                _ => Url::parse(DEFAULT_BASE_URL),
            }
            .expect("env variable `SENDIO_BASE_URL` should be a valid URL")
        });

        let env_user_agent = format!("sendio-rust:{}", env!("CARGO_PKG_VERSION"));

        // ==== Rate limiting is a non-blocking thing only ====
        #[cfg(not(feature = "blocking"))]
        let rate_limit_per_sec = env::var("SENDIO_RATE_LIMIT")
            .unwrap_or_else(|_| "9".to_owned())
            .parse::<u32>()
            .expect("env variable `SENDIO_RATE_LIMIT` should be a valid u32");

        #[cfg(not(feature = "blocking"))]
        let quota = Quota::with_period(Duration::from_millis(1100))
            .expect("Valid quota")
            .allow_burst(
                NonZeroU32::new(rate_limit_per_sec).expect("Rate limit is a valid non zero u32"),
            );

        #[cfg(not(feature = "blocking"))]
        let limiter = Arc::new(RateLimiter::direct_with_clock(quota, MonotonicClock));
        // ====================================================

        Self {
            user_agent: env_user_agent,
            api_key,
            base_url: env_base_url,
            client,
            #[cfg(not(feature = "blocking"))]
            limiter,
        }
    }

    /// Constructs a new [`RequestBuilder`].
    pub(crate) fn build(&self, method: Method, path: &str) -> RequestBuilder {
        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/').to_owned();
        url.set_path(&format!("{}/{}", base_path, path.trim_start_matches('/')));

        self.client
            .request(method, url)
            .bearer_auth(self.api_key.as_str())
            .header(USER_AGENT, self.user_agent.as_str())
    }

    #[allow(unreachable_pub)]
    #[maybe_async::maybe_async]
    pub async fn send(&self, request: RequestBuilder) -> Result<Response> {
        #[cfg(not(feature = "blocking"))]
        {
            let jitter =
                governor::Jitter::new(Duration::from_millis(10), Duration::from_millis(50));
            self.limiter.until_ready_with_jitter(jitter).await;
        }

        let request = request.build()?;

        let response = self.client.execute(request).await?;

        match response.status() {
            StatusCode::TOO_MANY_REQUESTS => {
                let headers = response.headers();

                let ratelimit_limit = headers
                    .get("ratelimit-limit")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());
                let ratelimit_remaining = headers
                    .get("ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());
                let ratelimit_reset = headers
                    .get("ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());

                Err(Error::RateLimit {
                    ratelimit_limit,
                    ratelimit_remaining,
                    ratelimit_reset,
                })
            }
            x if x.is_client_error() || x.is_server_error() => {
                // TODO: Make this more testable
                let content_type_is_html = response
                    .headers()
                    .get("content-type")
                    .and_then(|el| el.to_str().ok())
                    .is_some_and(|content_type| content_type.contains("html"));

                if content_type_is_html {
                    return Err(Error::Parse {
                        message: response.text().await?,
                        source: None,
                    });
                }

                let error_raw = response.text().await?;
                let error = serde_json::from_str::<ErrorResponse>(&error_raw).map_err(|e| {
                    Error::Parse {
                        message: error_raw,
                        source: Some(Box::new(e)),
                    }
                })?;

                Err(Error::Sendio(error))
            }
            _ => Ok(response),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Don't output API key.
        f.debug_struct("Client")
            .field("api_key", &"mx_*********")
            .field("user_agent", &self.user_agent.as_str())
            .field("base_url", &self.base_url.as_str())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;

    fn url_for(base_url: &str, path: &str) -> String {
        let config = Config::builder("mx_test")
            .base_url(base_url.parse().unwrap())
            .build();

        config
            .build(Method::GET, path)
            .build()
            .unwrap()
            .url()
            .to_string()
    }

    #[test]
    fn base_url_without_path() {
        assert_eq!(
            url_for("https://api.send.io", "/emails"),
            "https://api.send.io/emails"
        );
    }

    #[test]
    fn base_url_path_prefix_is_kept() {
        assert_eq!(
            url_for("https://proxy.example.com/sendio", "/emails/abc"),
            "https://proxy.example.com/sendio/emails/abc"
        );
        assert_eq!(
            url_for("https://proxy.example.com/sendio/", "/emails/abc"),
            "https://proxy.example.com/sendio/emails/abc"
        );
    }

    #[test]
    fn base_url_trailing_slash_is_optional() {
        assert_eq!(
            url_for("https://api.send.io/", "/emails"),
            "https://api.send.io/emails"
        );
        assert_eq!(
            url_for("https://api.send.io", "emails"),
            "https://api.send.io/emails"
        );
    }

    #[test]
    fn user_agent_is_sendio_rust() {
        let config = Config::builder("mx_test").build();
        let req = config.build(Method::GET, "/domains").build().unwrap();
        assert_eq!(
            req.headers().get(USER_AGENT).unwrap().to_str().unwrap(),
            format!("sendio-rust:{}", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(
            req.headers()
                .get(reqwest::header::AUTHORIZATION)
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer mx_test"
        );
    }

    #[test]
    #[serial_test::serial(env)]
    fn base_url_from_env_and_default() {
        // `unsafe_code` is forbidden, so the environment cannot be mutated here.
        // The builder value always wins over the environment.
        let config = Config::builder("mx_test")
            .base_url("http://localhost:9999/v1/".parse().unwrap())
            .build();
        assert_eq!(config.base_url.as_str(), "http://localhost:9999/v1/");

        if env::var("SENDIO_BASE_URL").map_or(true, |v| v.trim().is_empty()) {
            let config = Config::builder("mx_test").build();
            assert_eq!(config.base_url.as_str(), "https://api.send.io/");
        }
    }
}
