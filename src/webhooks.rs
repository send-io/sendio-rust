use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use reqwest::Method;
use sha2::Sha256;

use crate::{
    Config, Result,
    list_opts::{AfterPagination, ListOptions, ListResponse},
    types::{
        CreateWebhookOptions, CreateWebhookResponse, DeleteWebhookResponse,
        ReplayWebhookEventResponse, RotateWebhookSigningSecretResponse, UpdateWebhookOptions,
        UpdateWebhookResponse, VerifyWebhookOptions, Webhook, WebhookEventAttemptListResponse,
        WebhookEventDetails, WebhookEventListResponse, WebhookVerificationError,
    },
};

/// `Sendio` APIs for `/webhooks` endpoints.
///
/// To check the signature of an incoming webhook request, use
/// [`verify_webhook`](crate::verify_webhook).
#[derive(Clone, Debug)]
pub struct WebhookSvc(pub(crate) Arc<Config>);

impl WebhookSvc {
    /// Create a webhook to receive real-time notifications about email events.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn create(&self, webhook: CreateWebhookOptions) -> Result<CreateWebhookResponse> {
        let request = self.0.build(Method::POST, "/webhooks");
        let response = self.0.send(request.json(&webhook)).await?;
        let content = response.json::<CreateWebhookResponse>().await?;

        Ok(content)
    }

    /// Retrieve a single webhook for the authenticated user.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get(&self, webhook_id: &str) -> Result<Webhook> {
        let path = format!("/webhooks/{webhook_id}");

        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<Webhook>().await?;

        Ok(content)
    }

    /// Update an existing webhook configuration.
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        webhook_id: &str,
        update: UpdateWebhookOptions,
    ) -> Result<UpdateWebhookResponse> {
        let path = format!("/webhooks/{webhook_id}");

        let request = self.0.build(Method::PATCH, &path);
        let response = self.0.send(request.json(&update)).await?;
        let content = response.json::<UpdateWebhookResponse>().await?;

        Ok(content)
    }

    /// Retrieve a list of webhooks for the authenticated user.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list<T>(&self, list_opts: ListOptions<T>) -> Result<ListResponse<Webhook>> {
        let request = self.0.build(Method::GET, "/webhooks").query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<ListResponse<Webhook>>().await?;

        Ok(content)
    }

    /// Retrieve a list of events delivered to a webhook.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list_events<T: AfterPagination>(
        &self,
        webhook_id: &str,
        list_opts: ListOptions<T>,
    ) -> Result<WebhookEventListResponse> {
        let path = format!("/webhooks/{webhook_id}/events");
        let request = self.0.build(Method::GET, &path).query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<WebhookEventListResponse>().await?;

        Ok(content)
    }

    /// Retrieve a single event delivered to a webhook.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get_event(&self, webhook_id: &str, event_id: &str) -> Result<WebhookEventDetails> {
        let path = format!("/webhooks/{webhook_id}/events/{event_id}");
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<WebhookEventDetails>().await?;

        Ok(content)
    }

    /// Queue one more delivery of a webhook event to its webhook.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn replay_event(
        &self,
        webhook_id: &str,
        event_id: &str,
    ) -> Result<ReplayWebhookEventResponse> {
        let path = format!("/webhooks/{webhook_id}/events/{event_id}/replay");
        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<ReplayWebhookEventResponse>().await?;

        Ok(content)
    }

    /// Rotate the signing secret of a webhook. The previous secret keeps working for 24 hours.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn rotate_signing_secret(
        &self,
        webhook_id: &str,
    ) -> Result<RotateWebhookSigningSecretResponse> {
        let path = format!("/webhooks/{webhook_id}/signing-secret/rotate");
        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let content = response
            .json::<RotateWebhookSigningSecretResponse>()
            .await?;

        Ok(content)
    }

    /// Retrieve the delivery attempts for a webhook event.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list_event_attempts<T: AfterPagination>(
        &self,
        webhook_id: &str,
        event_id: &str,
        list_opts: ListOptions<T>,
    ) -> Result<WebhookEventAttemptListResponse> {
        let path = format!("/webhooks/{webhook_id}/events/{event_id}/attempts");
        let request = self.0.build(Method::GET, &path).query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<WebhookEventAttemptListResponse>().await?;

        Ok(content)
    }

    /// Remove an existing webhook.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn delete(&self, webhook_id: &str) -> Result<bool> {
        let path = format!("/webhooks/{webhook_id}");

        let request = self.0.build(Method::DELETE, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<DeleteWebhookResponse>().await?;

        Ok(content.deleted)
    }
}

/// Verify the signature of a webhook request sent by send.io.
///
/// Checks the `svix-id`, `svix-timestamp` and `svix-signature` headers against the raw
/// request body and the webhook's signing secret (`whsec_...`): the timestamp must be
/// within the tolerance (5 minutes unless changed with
/// [`VerifyWebhookOptions::with_tolerance`]) and one of the `v1` signatures must be the
/// HMAC-SHA256 of `{id}.{timestamp}.{payload}`. Signatures are compared in constant time.
///
/// Parse the payload with [`try_parse_event`] once it is verified.
///
/// ```
/// use sendio::types::{VerifyWebhookOptions, WebhookHeaders};
///
/// # fn handle(raw_body: &str, id: &str, timestamp: &str, signature: &str) {
/// let headers = WebhookHeaders { id, timestamp, signature };
/// let secret = std::env::var("SENDIO_WEBHOOK_SECRET").unwrap_or_default();
///
/// match sendio::verify_webhook(VerifyWebhookOptions::new(raw_body, headers, &secret)) {
///     Ok(()) => println!("{:?}", sendio::events::try_parse_event(raw_body)),
///     Err(err) => eprintln!("rejected webhook: {err}"),
/// }
/// # }
/// ```
///
/// This reads the system clock, which `wasm32-unknown-unknown` does not have.
///
/// [`try_parse_event`]: crate::events::try_parse_event
pub fn verify(
    options: VerifyWebhookOptions<'_>,
) -> std::result::Result<(), WebhookVerificationError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    verify_at(&options, now)
}

/// [`verify`] with the current Unix time passed in.
fn verify_at(
    options: &VerifyWebhookOptions<'_>,
    now: u64,
) -> std::result::Result<(), WebhookVerificationError> {
    let headers = &options.headers;
    if headers.id.is_empty() {
        return Err(WebhookVerificationError::MissingHeader("svix-id"));
    }
    if headers.timestamp.is_empty() {
        return Err(WebhookVerificationError::MissingHeader("svix-timestamp"));
    }
    if headers.signature.is_empty() {
        return Err(WebhookVerificationError::MissingHeader("svix-signature"));
    }

    let timestamp = headers
        .timestamp
        .trim()
        .parse::<u64>()
        .map_err(|_e| WebhookVerificationError::InvalidTimestamp)?;
    if now.abs_diff(timestamp) > options.tolerance.as_secs() {
        return Err(WebhookVerificationError::TimestampOutOfTolerance);
    }

    let secret = options
        .webhook_secret
        .strip_prefix("whsec_")
        .unwrap_or(options.webhook_secret);
    let key = BASE64
        .decode(secret)
        .map_err(|_e| WebhookVerificationError::InvalidSecret)?;
    if key.is_empty() {
        return Err(WebhookVerificationError::InvalidSecret);
    }

    let mut mac = Hmac::<Sha256>::new_from_slice(&key)
        .map_err(|_e| WebhookVerificationError::InvalidSecret)?;
    mac.update(headers.id.as_bytes());
    mac.update(b".");
    mac.update(headers.timestamp.as_bytes());
    mac.update(b".");
    mac.update(options.payload.as_bytes());

    // The header holds space-separated `version,signature` pairs, e.g. "v1,abc= v1,def=".
    let matched = headers
        .signature
        .split_whitespace()
        .filter_map(|entry| entry.strip_prefix("v1,"))
        .filter_map(|signature| BASE64.decode(signature).ok())
        .any(|signature| mac.clone().verify_slice(&signature).is_ok());

    if matched {
        Ok(())
    } else {
        Err(WebhookVerificationError::NoMatchingSignature)
    }
}

#[allow(unreachable_pub)]
pub mod types {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::events::EventType;

    /// How far a webhook's `svix-timestamp` may be from the current time by default.
    pub const DEFAULT_WEBHOOK_TOLERANCE: std::time::Duration = std::time::Duration::from_secs(300);

    /// The signature headers of a webhook request.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WebhookHeaders<'a> {
        /// The `svix-id` header.
        pub id: &'a str,
        /// The `svix-timestamp` header (Unix seconds).
        pub timestamp: &'a str,
        /// The `svix-signature` header: one or more space-separated `v1,<base64>` values.
        pub signature: &'a str,
    }

    /// Input for [`verify_webhook`](crate::verify_webhook).
    #[must_use]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VerifyWebhookOptions<'a> {
        /// The raw request body, exactly as received (not re-serialized JSON).
        pub payload: &'a str,
        /// The signature headers of the request.
        pub headers: WebhookHeaders<'a>,
        /// The webhook's signing secret (`whsec_...`).
        pub webhook_secret: &'a str,
        /// How far the timestamp may be from the current time, either way.
        pub tolerance: std::time::Duration,
    }

    impl<'a> VerifyWebhookOptions<'a> {
        /// Creates the options with the default tolerance of 5 minutes.
        pub const fn new(
            payload: &'a str,
            headers: WebhookHeaders<'a>,
            webhook_secret: &'a str,
        ) -> Self {
            Self {
                payload,
                headers,
                webhook_secret,
                tolerance: DEFAULT_WEBHOOK_TOLERANCE,
            }
        }

        /// Sets how far the timestamp may be from the current time, either way.
        pub const fn with_tolerance(mut self, tolerance: std::time::Duration) -> Self {
            self.tolerance = tolerance;
            self
        }
    }

    /// Why a webhook request failed verification.
    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum WebhookVerificationError {
        /// A signature header is missing or empty.
        #[error("{0} header is required")]
        MissingHeader(&'static str),
        /// The `svix-timestamp` header is not a Unix timestamp.
        #[error("invalid svix-timestamp header")]
        InvalidTimestamp,
        /// The timestamp is further from the current time than the tolerance allows.
        #[error("timestamp outside the tolerance window")]
        TimestampOutOfTolerance,
        /// The signing secret is empty or not valid base64.
        #[error("invalid webhook signing secret")]
        InvalidSecret,
        /// None of the `v1` signatures match the payload.
        #[error("no matching signature found")]
        NoMatchingSignature,
    }

    crate::define_id_type!(WebhookId);
    crate::define_id_type!(WebhookEventId);
    crate::define_id_type!(WebhookEventAttemptId);

    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum WebhookEventStatus {
        Pending,
        Attempting,
        Success,
        Failed,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebhookEvent {
        pub id: WebhookEventId,
        #[serde(rename = "type")]
        pub event_type: String,
        pub created_at: String,
        pub status: WebhookEventStatus,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebhookEventListResponse {
        pub object: String,
        pub has_more: bool,
        pub data: Vec<WebhookEvent>,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebhookEventDetails {
        pub object: String,
        pub id: WebhookEventId,
        #[serde(rename = "type")]
        pub event_type: String,
        pub created_at: String,
        pub status: WebhookEventStatus,
        pub next_attempt_at: Option<String>,
        pub payload: Value,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReplayWebhookEventResponse {
        pub object: String,
        pub id: WebhookEventId,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebhookEventAttempt {
        pub id: WebhookEventAttemptId,
        pub http_status_code: u16,
        pub response: String,
        pub sent_at: String,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebhookEventAttemptListResponse {
        pub object: String,
        pub has_more: bool,
        pub data: Vec<WebhookEventAttempt>,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    pub struct CreateWebhookOptions {
        endpoint: String,
        events: Vec<EventType>,
    }

    impl CreateWebhookOptions {
        pub fn new(
            endpoint: impl Into<String>,
            events: impl IntoIterator<Item = impl Into<EventType>>,
        ) -> Self {
            Self {
                endpoint: endpoint.into(),
                events: events.into_iter().map(Into::into).collect::<Vec<_>>(),
            }
        }
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateWebhookResponse {
        pub id: WebhookId,
        pub signing_secret: String,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RotateWebhookSigningSecretResponse {
        pub object: String,
        pub id: WebhookId,
        pub signing_secret: String,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Webhook {
        pub id: WebhookId,
        /// RFC 3339 in UTC, for example `2026-11-10T14:40:35.840078+00:00`.
        pub created_at: String,
        pub status: String,
        pub endpoint: String,
        #[serde(default)]
        pub events: Vec<EventType>,
        /// The signing secret. Left out for workspace Members.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub signing_secret: Option<String>,
        /// `true` when the signing secret is left out because the caller is a workspace Member.
        #[serde(default)]
        pub signing_secret_hidden: bool,
        /// Why send.io disabled the endpoint (after 3 days of failed deliveries).
        #[serde(default)]
        pub disabled_reason: Option<String>,
        /// When the current run of failed deliveries started (RFC 3339).
        #[serde(default)]
        pub failing_since: Option<String>,
    }

    #[derive(Debug, Clone, Copy, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum WebhookStatus {
        Enabled,
        Disabled,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Default)]
    pub struct UpdateWebhookOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        events: Option<Vec<EventType>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<WebhookStatus>,
    }

    impl UpdateWebhookOptions {
        #[inline]
        pub fn with_endpoint(mut self, endpoint: &str) -> Self {
            self.endpoint = Some(endpoint.to_owned());
            self
        }

        #[inline]
        pub fn with_events(
            mut self,
            events: impl IntoIterator<Item = impl Into<EventType>>,
        ) -> Self {
            self.events = Some(events.into_iter().map(Into::into).collect());
            self
        }

        #[inline]
        pub fn with_status(mut self, status: WebhookStatus) -> Self {
            self.status = Some(status);
            self
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateWebhookResponse {
        /// Unique identifier for the updated webhook.
        pub id: WebhookId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeleteWebhookResponse {
        /// The ID of the webhook.
        #[allow(dead_code)]
        pub id: WebhookId,
        /// Indicates whether the webhook was deleted successfully.
        pub deleted: bool,
    }
}

#[cfg(test)]
mod test {
    use crate::{
        events::EmailEventType,
        list_opts::ListOptions,
        types::{CreateWebhookOptions, CreateWebhookResponse},
        types::{
            RotateWebhookSigningSecretResponse, Webhook, WebhookEventAttemptListResponse,
            WebhookEventDetails, WebhookEventListResponse,
        },
    };
    #[cfg(not(feature = "blocking"))]
    use crate::{
        test::{CLIENT, DebugResult},
        types::{UpdateWebhookOptions, WebhookStatus},
    };

    #[tokio_shared_rt::test(shared = true)]
    #[serial_test::serial]
    #[cfg(not(feature = "blocking"))]
    #[ignore = "live API test; run with `cargo test -- --ignored` and SENDIO_API_KEY set"]
    async fn all() -> DebugResult<()> {
        let sendio = &*CLIENT;

        let events = [
            EmailEventType::EmailSent,
            EmailEventType::EmailDelivered,
            EmailEventType::EmailBounced,
        ];
        let webhook = CreateWebhookOptions::new("https://webhook.example.com/handler", events);
        let id = sendio.webhooks.create(webhook).await?.id;

        std::thread::sleep(std::time::Duration::from_secs(1));

        let webhook = sendio.webhooks.get(&id).await?;
        assert_eq!(webhook.events.len(), 3);
        let webhooks = sendio.webhooks.list(ListOptions::default()).await?;
        assert!(!webhooks.is_empty());

        let update = UpdateWebhookOptions::default()
            .with_endpoint("https://new-webhook.example.com/handler")
            .with_events([EmailEventType::EmailSent, EmailEventType::EmailDelivered])
            .with_status(WebhookStatus::Enabled);
        let _webhook = sendio.webhooks.update(&id, update).await?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let webhook = sendio.webhooks.get(&id).await?;
        assert_eq!(webhook.events.len(), 2);

        let deleted = sendio.webhooks.delete(&id).await?;
        assert!(deleted);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let webhooks = sendio.webhooks.list(ListOptions::default()).await?;
        assert!(webhooks.is_empty());

        Ok(())
    }

    #[test]
    fn serialize_test() {
        let events = [EmailEventType::EmailSent];

        let webhook =
            CreateWebhookOptions::new("https://webhook.example.com/handler".to_owned(), events);

        let res = serde_json::to_string(&webhook);
        assert!(res.is_ok());
    }

    #[test]
    fn deserialize_test() {
        let create_webhook_res = r#"{
  "object": "webhook",
  "id": "4dd369bc-aa82-4ff3-97de-514ae3000ee0",
  "signing_secret": "whsec_xxxxxxxxxx"
}"#;

        let res = serde_json::from_str::<CreateWebhookResponse>(create_webhook_res);
        assert!(res.is_ok());

        let webhook = r#"{
  "object": "webhook",
  "id": "4dd369bc-aa82-4ff3-97de-514ae3000ee0",
  "created_at": "2023-08-22T15:28:00+00:00",
  "status": "enabled",
  "endpoint": "https://webhook.example.com/handler",
  "events": ["email.sent", "email.received"],
  "signing_secret": "whsec_xxxxxxxxxx"
}"#;

        let res = serde_json::from_str::<Webhook>(webhook);
        assert!(res.is_ok());
    }

    // SENDIO-337: a webhook send.io disabled, read by a workspace Member.
    #[test]
    fn deserialize_disabled_webhook_as_member() -> serde_json::Result<()> {
        let webhook = r#"{
  "object": "webhook",
  "id": "4dd369bc-aa82-4ff3-97de-514ae3000ee0",
  "created_at": "2026-11-01T09:00:00.123456+00:00",
  "status": "disabled",
  "disabled_reason": "Deliveries failed for 3 days.",
  "failing_since": "2026-11-07T10:15:00+00:00",
  "endpoint": "https://webhook.example.com/handler",
  "events": ["email.sent"],
  "signing_secret_hidden": true
}"#;

        let webhook = serde_json::from_str::<Webhook>(webhook)?;

        assert_eq!(webhook.status, "disabled");
        assert_eq!(
            webhook.disabled_reason.as_deref(),
            Some("Deliveries failed for 3 days.")
        );
        assert_eq!(
            webhook.failing_since.as_deref(),
            Some("2026-11-07T10:15:00+00:00")
        );
        assert!(webhook.signing_secret_hidden);
        assert!(webhook.signing_secret.is_none());

        Ok(())
    }

    #[test]
    fn deserialize_rotate_signing_secret_response() -> serde_json::Result<()> {
        let rotated = r#"{
  "object": "webhook",
  "id": "4dd369bc-aa82-4ff3-97de-514ae3000ee0",
  "signing_secret": "whsec_yyyyyyyyyy"
}"#;

        let rotated = serde_json::from_str::<RotateWebhookSigningSecretResponse>(rotated)?;

        assert_eq!(rotated.id.as_ref(), "4dd369bc-aa82-4ff3-97de-514ae3000ee0");
        assert_eq!(rotated.signing_secret, "whsec_yyyyyyyyyy");

        Ok(())
    }

    #[test]
    fn deserialize_webhook_event_responses() -> serde_json::Result<()> {
        let events = r#"{
  "object": "list",
  "has_more": false,
  "data": [{
    "id": "msg_1srOrx2ZWZBpBUvZwXKQmoEYga2",
    "type": "email.sent",
    "created_at": "2026-08-22T15:28:00.000Z",
    "status": "success"
  }]
}"#;
        let event = r#"{
  "object": "webhook_event",
  "id": "msg_1srOrx2ZWZBpBUvZwXKQmoEYga2",
  "type": "email.sent",
  "created_at": "2026-08-22T15:28:00.000Z",
  "status": "attempting",
  "next_attempt_at": null,
  "payload": {"type": "email.sent", "data": {"email_id": "abc"}}
}"#;
        let attempts = r#"{
  "object": "list",
  "has_more": false,
  "data": [{
    "id": "atmpt_2ZbUCwvGmIT4mLIN6d3Yz0Ainbd",
    "http_status_code": 200,
    "response": "{\"ok\":true}",
    "sent_at": "2026-08-22T15:28:05.000Z"
  }]
}"#;

        let events = serde_json::from_str::<WebhookEventListResponse>(events)?;
        let event = serde_json::from_str::<WebhookEventDetails>(event)?;
        let attempts = serde_json::from_str::<WebhookEventAttemptListResponse>(attempts)?;

        assert_eq!(
            events.data.first().map(|event| event.id.as_ref()),
            Some("msg_1srOrx2ZWZBpBUvZwXKQmoEYga2")
        );
        assert!(event.next_attempt_at.is_none());
        assert_eq!(
            event.payload.get("type"),
            Some(&serde_json::json!("email.sent"))
        );
        assert_eq!(
            attempts.data.first().map(|attempt| attempt.id.as_ref()),
            Some("atmpt_2ZbUCwvGmIT4mLIN6d3Yz0Ainbd")
        );

        Ok(())
    }

    #[test]
    fn serialize_after_only_pagination() -> serde_json::Result<()> {
        let options = ListOptions::default()
            .with_limit(10)
            .list_after("msg_1srOrx2ZWZBpBUvZwXKQmoEYga2");
        let query = serde_json::to_value(options)?;

        assert_eq!(query.get("limit"), Some(&serde_json::json!(10)));
        assert_eq!(
            query.get("after"),
            Some(&serde_json::json!("msg_1srOrx2ZWZBpBUvZwXKQmoEYga2"))
        );
        assert_eq!(query.get("before"), Some(&serde_json::Value::Null));

        Ok(())
    }
}

#[cfg(test)]
mod verify_test {
    use std::time::Duration;

    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    use super::verify_at;
    use crate::types::{VerifyWebhookOptions, WebhookHeaders, WebhookVerificationError};

    const SECRET: &str = "whsec_dGVzdC1zZWNyZXQtMDEyMzQ1Njc4OWFiY2RlZg==";
    const OTHER_SECRET: &str = "whsec_b3RoZXItc2VjcmV0";
    const PAYLOAD: &str =
        r#"{"type":"email.delivered","data":{"email_id":"4ef9a417-02e9-4d39-ad75-9611e0fcc33c"}}"#;
    const ID: &str = "msg_2abc";
    const NOW: u64 = 1_791_450_446;

    /// Signs `{id}.{timestamp}.{payload}` the way send.io does.
    #[allow(clippy::unwrap_used)]
    fn sign(secret: &str, timestamp: &str, payload: &str) -> String {
        let key = BASE64.decode(secret.trim_start_matches("whsec_")).unwrap();
        let mut mac = Hmac::<Sha256>::new_from_slice(&key).unwrap();
        mac.update(format!("{ID}.{timestamp}.{payload}").as_bytes());
        format!("v1,{}", BASE64.encode(mac.finalize().into_bytes()))
    }

    fn check(
        payload: &str,
        timestamp: &str,
        signature: &str,
        now: u64,
    ) -> Result<(), WebhookVerificationError> {
        let headers = WebhookHeaders {
            id: ID,
            timestamp,
            signature,
        };
        verify_at(&VerifyWebhookOptions::new(payload, headers, SECRET), now)
    }

    #[test]
    fn accepts_svix_reference_vector() {
        // From the Svix guide to verifying payloads manually.
        let headers = WebhookHeaders {
            id: "msg_p5jXN8AQM9LWM0D4loKWxJek",
            timestamp: "1614265330",
            signature: "v1,g0hM9SsE+OTPJTGt/tmIKtSyZlE3uFJELVlNIOLJ1OE=",
        };
        let options = VerifyWebhookOptions::new(
            r#"{"test": 2432232314}"#,
            headers,
            "whsec_MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw",
        );
        assert_eq!(verify_at(&options, 1_614_265_330), Ok(()));
    }

    #[test]
    fn accepts_a_signed_payload() {
        let ts = NOW.to_string();
        let signature = sign(SECRET, &ts, PAYLOAD);
        assert_eq!(check(PAYLOAD, &ts, &signature, NOW), Ok(()));
    }

    #[test]
    fn accepts_any_of_several_signatures() {
        let ts = NOW.to_string();
        let other = sign(OTHER_SECRET, &ts, PAYLOAD);
        let good = sign(SECRET, &ts, PAYLOAD);
        let header = format!("v2,ignored {other} {good}");
        assert_eq!(check(PAYLOAD, &ts, &header, NOW), Ok(()));
    }

    #[test]
    fn accepts_a_secret_without_prefix() {
        let ts = NOW.to_string();
        let signature = sign(SECRET, &ts, PAYLOAD);
        let headers = WebhookHeaders {
            id: ID,
            timestamp: &ts,
            signature: &signature,
        };
        let bare = SECRET.trim_start_matches("whsec_");
        let options = VerifyWebhookOptions::new(PAYLOAD, headers, bare);
        assert_eq!(verify_at(&options, NOW), Ok(()));
    }

    #[test]
    fn rejects_a_tampered_payload() {
        let ts = NOW.to_string();
        let signature = sign(SECRET, &ts, PAYLOAD);
        let tampered = PAYLOAD.replace("delivered", "bounced");
        assert_eq!(
            check(&tampered, &ts, &signature, NOW),
            Err(WebhookVerificationError::NoMatchingSignature)
        );
    }

    #[test]
    fn rejects_a_signature_made_with_another_secret() {
        let ts = NOW.to_string();
        let signature = sign(OTHER_SECRET, &ts, PAYLOAD);
        assert_eq!(
            check(PAYLOAD, &ts, &signature, NOW),
            Err(WebhookVerificationError::NoMatchingSignature)
        );
    }

    #[test]
    fn rejects_expired_and_future_timestamps() {
        let old = (NOW - 301).to_string();
        let signature = sign(SECRET, &old, PAYLOAD);
        assert_eq!(
            check(PAYLOAD, &old, &signature, NOW),
            Err(WebhookVerificationError::TimestampOutOfTolerance)
        );

        let future = (NOW + 301).to_string();
        let signature = sign(SECRET, &future, PAYLOAD);
        assert_eq!(
            check(PAYLOAD, &future, &signature, NOW),
            Err(WebhookVerificationError::TimestampOutOfTolerance)
        );

        let edge = (NOW - 300).to_string();
        let signature = sign(SECRET, &edge, PAYLOAD);
        assert_eq!(check(PAYLOAD, &edge, &signature, NOW), Ok(()));
    }

    #[test]
    fn tolerance_is_configurable() {
        let old = (NOW - 3600).to_string();
        let signature = sign(SECRET, &old, PAYLOAD);
        let headers = WebhookHeaders {
            id: ID,
            timestamp: &old,
            signature: &signature,
        };
        let options = VerifyWebhookOptions::new(PAYLOAD, headers, SECRET);
        assert_eq!(
            verify_at(&options, NOW),
            Err(WebhookVerificationError::TimestampOutOfTolerance)
        );
        let options = options.with_tolerance(Duration::from_secs(3600));
        assert_eq!(verify_at(&options, NOW), Ok(()));
    }

    #[test]
    fn rejects_missing_and_malformed_input() {
        let ts = NOW.to_string();
        let signature = sign(SECRET, &ts, PAYLOAD);
        assert_eq!(
            check(PAYLOAD, "", &signature, NOW),
            Err(WebhookVerificationError::MissingHeader("svix-timestamp"))
        );
        assert_eq!(
            check(PAYLOAD, &ts, "", NOW),
            Err(WebhookVerificationError::MissingHeader("svix-signature"))
        );
        assert_eq!(
            check(PAYLOAD, "yesterday", &signature, NOW),
            Err(WebhookVerificationError::InvalidTimestamp)
        );
        assert_eq!(
            check(PAYLOAD, &ts, "v1,not-base64!!", NOW),
            Err(WebhookVerificationError::NoMatchingSignature)
        );

        let headers = WebhookHeaders {
            id: "",
            timestamp: &ts,
            signature: &signature,
        };
        let options = VerifyWebhookOptions::new(PAYLOAD, headers, SECRET);
        assert_eq!(
            verify_at(&options, NOW),
            Err(WebhookVerificationError::MissingHeader("svix-id"))
        );

        let headers = WebhookHeaders {
            id: ID,
            timestamp: &ts,
            signature: &signature,
        };
        for secret in ["", "whsec_", "whsec_%%%"] {
            let options = VerifyWebhookOptions::new(PAYLOAD, headers, secret);
            assert_eq!(
                verify_at(&options, NOW),
                Err(WebhookVerificationError::InvalidSecret)
            );
        }
    }

    #[test]
    fn verify_reads_the_clock() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        let ts = now.to_string();
        let signature = sign(SECRET, &ts, PAYLOAD);
        let headers = WebhookHeaders {
            id: ID,
            timestamp: &ts,
            signature: &signature,
        };
        let options = VerifyWebhookOptions::new(PAYLOAD, headers, SECRET);
        assert_eq!(crate::verify_webhook(options), Ok(()));
    }
}
