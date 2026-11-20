use std::sync::Arc;
use std::{env, fmt};

#[cfg(not(feature = "blocking"))]
use reqwest::Client as ReqwestClient;
#[cfg(feature = "blocking")]
use reqwest::blocking::Client as ReqwestClient;

use crate::{
    batch::BatchSvc,
    config::Config,
    events::EventsSvc,
    logs::LogsSvc,
    mailboxes::MailboxesSvc,
    oauth::OAuthSvc,
    services::{AutomationsSvc, ReceivingSvc, SuppressionsSvc},
    webhooks::WebhookSvc,
};
use crate::{
    services::{
        ApiKeysSvc, BroadcastsSvc, ContactsSvc, DomainsSvc, EmailsSvc, SegmentsSvc, TemplateSvc,
    },
    topics::TopicsSvc,
    transactional::TransactionalSvc,
    usage::UsageSvc,
};

#[cfg(doc)]
use crate::ConfigBuilder;

/// The [Sendio](https://send.io) client.
#[must_use]
#[derive(Clone)]
pub struct Sendio {
    /// `Sendio` APIs for `/emails` endpoints.
    pub emails: EmailsSvc,
    /// `Sendio` APIs for the batch `/emails` endpoints.
    pub batch: BatchSvc,
    /// `Sendio` APIs for `/api-keys` endpoints.
    pub api_keys: ApiKeysSvc,
    /// `Sendio` APIs for `/audiences` endpoints.
    pub segments: SegmentsSvc,
    /// `Sendio` APIs for `/audiences/:id/contacts` endpoints.
    pub contacts: ContactsSvc,
    /// `Sendio` APIs for `/domains` endpoints.
    pub domains: DomainsSvc,
    /// `Sendio` APIs for `/broadcasts` endpoints.
    pub broadcasts: BroadcastsSvc,
    /// `Sendio` APIs for `/templates` endpoints.
    pub templates: TemplateSvc,
    /// `Sendio` APIs for `/topics` endpoints.
    pub topics: TopicsSvc,
    /// `Sendio` APIs for `/transactional` endpoints.
    pub transactional: TransactionalSvc,
    /// `Sendio` APIs for `/emails/receiving` endpoints.
    pub receiving: ReceivingSvc,
    /// `Sendio` APIs for `/webhooks` endpoints.
    pub webhooks: WebhookSvc,
    /// `Sendio` APIs for `/logs` endpoints.
    pub logs: LogsSvc,
    /// `Sendio` APIs for `/automations` endpoints.
    pub automations: AutomationsSvc,
    /// `Sendio` APIs for `/events` endpoints.
    pub events: EventsSvc,
    /// `Sendio` APIs for `/oauth` endpoints.
    pub oauth: OAuthSvc,
    /// `Sendio` APIs for `/suppressions` endpoints.
    pub suppressions: SuppressionsSvc,
    /// `Sendio` APIs for `/usage` endpoints.
    pub usage: UsageSvc,
    /// `Sendio` APIs for `/mailboxes` endpoints.
    pub mailboxes: MailboxesSvc,
}

impl Sendio {
    /// Creates a new [`Sendio`] client.
    ///
    /// ### Panics
    ///
    /// - Panics if the environment variable `SENDIO_BASE_URL` is set but is not a valid `URL`.
    ///
    /// [`Sendio`]: https://send.io
    pub fn new(api_key: &str) -> Self {
        Self::with_client(api_key, ReqwestClient::default())
    }

    /// Creates a new [`Sendio`] client with a provided [`reqwest::Client`].
    ///
    /// ### Panics
    ///
    /// - Panics if the environment variable `SENDIO_BASE_URL` is set but is not a valid `URL`.
    ///
    /// [`Sendio`]: https://send.io
    /// [`reqwest::Client`]: ReqwestClient
    pub fn with_client(api_key: &str, client: ReqwestClient) -> Self {
        let config = Config::new(api_key.to_owned(), client, None);
        Self::with_config(config)
    }

    /// Creates a new [`Sendio`] client with a provided [`Config`].
    ///
    /// Use [`ConfigBuilder::new`] to construct a [`Config`] instance.
    ///
    /// ### Panics
    ///
    /// -   Panics if the base url has not been set with [`ConfigBuilder::base_url`]
    ///     and the environment variable `SENDIO_BASE_URL` _is_ set but is not a valid `URL`.
    ///
    /// [`Sendio`]: https://send.io
    /// [`reqwest::Client`]: ReqwestClient
    pub fn with_config(config: Config) -> Self {
        let inner = Arc::new(config);
        Self {
            api_keys: ApiKeysSvc(Arc::clone(&inner)),
            segments: SegmentsSvc(Arc::clone(&inner)),
            contacts: ContactsSvc(Arc::clone(&inner)),
            domains: DomainsSvc(Arc::clone(&inner)),
            emails: EmailsSvc(Arc::clone(&inner)),
            batch: BatchSvc(Arc::clone(&inner)),
            broadcasts: BroadcastsSvc(Arc::clone(&inner)),
            templates: TemplateSvc(Arc::clone(&inner)),
            topics: TopicsSvc(Arc::clone(&inner)),
            transactional: TransactionalSvc(Arc::clone(&inner)),
            receiving: ReceivingSvc(Arc::clone(&inner)),
            webhooks: WebhookSvc(Arc::clone(&inner)),
            logs: LogsSvc(Arc::clone(&inner)),
            automations: AutomationsSvc::new(Arc::clone(&inner)),
            events: EventsSvc(Arc::clone(&inner)),
            oauth: OAuthSvc(Arc::clone(&inner)),
            suppressions: SuppressionsSvc(Arc::clone(&inner)),
            mailboxes: MailboxesSvc(Arc::clone(&inner)),
            usage: UsageSvc(inner),
        }
    }

    /// Returns the reference to the used `User-Agent` header value.
    #[inline]
    #[must_use]
    pub fn user_agent(&self) -> &str {
        self.config().user_agent.as_str()
    }

    /// Returns the reference to the provided `API key`.
    #[inline]
    #[must_use]
    pub fn api_key(&self) -> &str {
        self.config().api_key.as_ref()
    }

    /// Returns the reference to the used `base URL`.
    ///
    /// ### Notes
    ///
    /// Use the `SENDIO_BASE_URL` environment variable to override.
    #[inline]
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.config().base_url.as_str()
    }

    /// Returns the underlying [`reqwest::Client`].
    ///
    /// [`reqwest::Client`]: ReqwestClient
    #[inline]
    #[must_use]
    pub fn client(&self) -> ReqwestClient {
        self.config().client.clone()
    }

    /// Returns the reference to the inner [`Config`].
    #[inline]
    fn config(&self) -> &Config {
        &self.emails.0
    }

    /// Send a raw request to any API endpoint and get back a [`serde_json::Value`].
    /// Useful as a fallback when parsing fails.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use sendio::Sendio;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let sendio = Sendio::default();
    ///
    ///   let query = sendio::json!({
    ///       "limit": 10,
    ///   });
    ///
    ///   let raw_response = sendio
    ///     .send_raw(
    ///       sendio::Method::GET,
    ///       "/emails",
    ///       Some(query),
    ///       None::<()>, // The turbofish is needed because of the `impl`
    ///       None,
    ///     )
    ///     .await
    ///     .unwrap();
    ///
    ///   assert!(raw_response.get("data").is_some());
    ///   let data = raw_response.get("data").and_then(|v| v.as_array()).unwrap();
    ///   assert!(!data.is_empty());
    /// }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn send_raw(
        &self,
        method: reqwest::Method,
        path: &str,
        query: Option<impl serde::Serialize>,
        body: Option<impl serde::Serialize>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> crate::Result<serde_json::Value> {
        let mut request = self.config().build(method, path);

        if let Some(q) = query {
            request = request.query(&q);
        }

        if let Some(h) = headers {
            request = request.headers(h);
        }

        if let Some(b) = body {
            request = request.json(&b);
        }

        let response = self.config().send(request).await?;
        let value = response.json::<serde_json::Value>().await?;

        Ok(value)
    }
}

impl Default for Sendio {
    /// Creates a new [`Sendio`] client from the `SENDIO_API_KEY` environment variable .
    ///
    /// ### Panics
    ///
    /// - Panics if the environment variable `SENDIO_API_KEY` is not set.
    /// - Panics if the environment variable `SENDIO_BASE_URL` is set but is not a valid `URL`.
    fn default() -> Self {
        let api_key = env::var("SENDIO_API_KEY")
            .expect("env variable `SENDIO_API_KEY` should be a valid API key");

        Self::new(api_key.as_str())
    }
}

impl fmt::Debug for Sendio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.emails, f)
    }
}
