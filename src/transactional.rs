use std::sync::Arc;

use reqwest::Method;

use crate::{
    Config, Result,
    idempotent::Idempotent,
    list_opts::ListResponse,
    types::{
        CreateEmailResponse, CreateTransactionalEmailOptions, DeleteTransactionalEmailResponse,
        ListTransactionalEmailsOptions, SendTestTransactionalEmailOptions, TransactionalEmail,
        TransactionalEmailStats, TransactionalEmailStatsOptions, UpdateTransactionalEmailOptions,
    },
};

/// `Sendio` APIs for `/transactional` endpoints.
///
/// A transactional email is a named, saved send of a published template: a sign-in code, a
/// password reset, a receipt. It stores the envelope (From, subject, preview text, reply-to, and
/// open and click tracking, off by default) and a template; the content is always the template's
/// published version at send time. Send it with
/// [`CreateEmailBaseOptions::transactional`](crate::types::CreateEmailBaseOptions::transactional),
/// passing only the recipients and the variables. A republish of the template, or an update,
/// reaches every send within 10 seconds.
#[derive(Clone, Debug)]
pub struct TransactionalSvc(pub(crate) Arc<Config>);

impl TransactionalSvc {
    /// Create a transactional email.
    ///
    /// Add an `Idempotency-Key` with
    /// [`CreateTransactionalEmailOptions::with_idempotency_key`]: a retry with the same key
    /// returns the first answer instead of a second transactional email.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        options: impl Into<Idempotent<CreateTransactionalEmailOptions>>,
    ) -> Result<TransactionalEmail> {
        let options: Idempotent<CreateTransactionalEmailOptions> = options.into();

        let mut request = self.0.build(Method::POST, "/transactional");

        if let Some(ref idempotency_key) = options.idempotency_key {
            request = request.header("Idempotency-Key", idempotency_key);
        }

        let response = self.0.send(request.json(&options)).await?;
        let content = response.json::<TransactionalEmail>().await?;

        Ok(content)
    }

    /// Retrieve transactional emails, newest first, optionally filtered by name or subject
    /// (`search`) and by template.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list<T>(
        &self,
        list_opts: impl Into<ListTransactionalEmailsOptions<T>>,
    ) -> Result<ListResponse<TransactionalEmail>> {
        let list_opts = list_opts.into();
        let request = self
            .0
            .build(Method::GET, "/transactional")
            .query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<ListResponse<TransactionalEmail>>().await?;

        Ok(content)
    }

    /// Retrieve a transactional email, with the variables of the template version its sends
    /// use now.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get(&self, transactional_id: &str) -> Result<TransactionalEmail> {
        let path = format!("/transactional/{transactional_id}");

        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<TransactionalEmail>().await?;

        Ok(content)
    }

    /// Change a transactional email. Only the fields that are set are sent; sends use the
    /// change within 10 seconds.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        transactional_id: &str,
        update: UpdateTransactionalEmailOptions,
    ) -> Result<TransactionalEmail> {
        let path = format!("/transactional/{transactional_id}");

        let request = self.0.build(Method::PATCH, &path);
        let response = self.0.send(request.json(&update)).await?;
        let content = response.json::<TransactionalEmail>().await?;

        Ok(content)
    }

    /// Delete a transactional email; sends of it then fail with 404. Its stats stay readable.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn delete(&self, transactional_id: &str) -> Result<DeleteTransactionalEmailResponse> {
        let path = format!("/transactional/{transactional_id}");

        let request = self.0.build(Method::DELETE, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<DeleteTransactionalEmailResponse>().await?;

        Ok(content)
    }

    /// Send a transactional email to 1 to 5 addresses now, with `[Test] ` before the subject.
    /// Tests are not counted in stats.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn send_test(
        &self,
        transactional_id: &str,
        test: SendTestTransactionalEmailOptions,
    ) -> Result<CreateEmailResponse> {
        let path = format!("/transactional/{transactional_id}/test");

        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request.json(&test)).await?;
        let content = response.json::<CreateEmailResponse>().await?;

        Ok(content)
    }

    /// Retrieve a transactional email's numbers in a period (the last 30 days by default).
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn stats(
        &self,
        transactional_id: &str,
        options: TransactionalEmailStatsOptions,
    ) -> Result<TransactionalEmailStats> {
        let path = format!("/transactional/{transactional_id}/stats");

        let request = self.0.build(Method::GET, &path).query(&options);
        let response = self.0.send(request).await?;
        let content = response.json::<TransactionalEmailStats>().await?;

        Ok(content)
    }
}

#[allow(unreachable_pub)]
pub mod types {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::{
        idempotent::Idempotent,
        list_opts::{ListAfter, ListBefore, ListOptions, TimeNotSpecified},
        types::TemplateId,
    };

    crate::define_id_type!(TransactionalEmailId);

    /// The details of a new transactional email.
    ///
    /// See [relevant docs].
    ///
    /// [relevant docs]: <https://developer.send.io/docs/api>
    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    pub struct CreateTransactionalEmailOptions {
        name: String,
        template_id: String,
        from: String,
        subject: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        preview_text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        track_opens: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        track_clicks: Option<bool>,
    }

    impl CreateTransactionalEmailOptions {
        /// Creates a new [`CreateTransactionalEmailOptions`].
        ///
        /// - `name`: 1 to 255 characters, shown in the dashboard.
        /// - `template_id`: The id or alias of a published template.
        /// - `from`: The sender, on a verified sending domain, e.g.
        ///   `"Acme <security@yourdomain.com>"`.
        /// - `subject`: The subject; it may use the template's variables, e.g.
        ///   `"Your code is {{CODE}}"`.
        pub fn new(
            name: impl Into<String>,
            template_id: impl Into<String>,
            from: impl Into<String>,
            subject: impl Into<String>,
        ) -> Self {
            Self {
                name: name.into(),
                template_id: template_id.into(),
                from: from.into(),
                subject: subject.into(),
                preview_text: None,
                reply_to: None,
                track_opens: None,
                track_clicks: None,
            }
        }

        /// Sets the preview text shown by email clients next to the subject.
        #[inline]
        pub fn with_preview_text(mut self, preview_text: &str) -> Self {
            self.preview_text = Some(preview_text.to_owned());
            self
        }

        /// Adds a reply-to address (at most 50).
        #[inline]
        pub fn with_reply(mut self, to: &str) -> Self {
            let reply_to = self.reply_to.get_or_insert_with(Vec::new);
            reply_to.push(to.to_owned());
            self
        }

        /// Turns open tracking on or off (off by default; usually off for codes and
        /// verification links).
        #[inline]
        pub const fn with_track_opens(mut self, track_opens: bool) -> Self {
            self.track_opens = Some(track_opens);
            self
        }

        /// Turns click tracking on or off (off by default; usually off for codes and
        /// verification links).
        #[inline]
        pub const fn with_track_clicks(mut self, track_clicks: bool) -> Self {
            self.track_clicks = Some(track_clicks);
            self
        }

        /// Adds an `Idempotency-Key` header to the request: a retry with the same key returns
        /// the first answer instead of a second transactional email.
        #[inline]
        pub fn with_idempotency_key(self, idempotency_key: &str) -> Idempotent<Self> {
            Idempotent {
                idempotency_key: Some(idempotency_key.to_owned()),
                data: self,
            }
        }
    }

    /// Changes to a [`TransactionalEmail`]. Only the fields that are set are sent.
    #[must_use]
    #[derive(Debug, Default, Clone, Serialize)]
    pub struct UpdateTransactionalEmailOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        template_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subject: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        preview_text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        track_opens: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        track_clicks: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_updated_at: Option<String>,
    }

    impl UpdateTransactionalEmailOptions {
        /// Creates a new, empty [`UpdateTransactionalEmailOptions`].
        pub fn new() -> Self {
            Self::default()
        }

        /// Sets a new name.
        #[inline]
        pub fn with_name(mut self, name: &str) -> Self {
            self.name = Some(name.to_owned());
            self
        }

        /// Points the transactional email at another published template (id or alias).
        #[inline]
        pub fn with_template_id(mut self, template_id: &str) -> Self {
            self.template_id = Some(template_id.to_owned());
            self
        }

        /// Sets a new sender, on a verified sending domain.
        #[inline]
        pub fn with_from(mut self, from: &str) -> Self {
            self.from = Some(from.to_owned());
            self
        }

        /// Sets a new subject.
        #[inline]
        pub fn with_subject(mut self, subject: &str) -> Self {
            self.subject = Some(subject.to_owned());
            self
        }

        /// Sets the preview text; an empty string clears it.
        #[inline]
        pub fn with_preview_text(mut self, preview_text: &str) -> Self {
            self.preview_text = Some(preview_text.to_owned());
            self
        }

        /// Replaces the reply-to addresses; an empty slice clears them.
        #[inline]
        pub fn with_reply_to(mut self, reply_to: &[&str]) -> Self {
            self.reply_to = Some(reply_to.iter().map(|&to| to.to_owned()).collect());
            self
        }

        /// Turns open tracking on or off.
        #[inline]
        pub const fn with_track_opens(mut self, track_opens: bool) -> Self {
            self.track_opens = Some(track_opens);
            self
        }

        /// Turns click tracking on or off.
        #[inline]
        pub const fn with_track_clicks(mut self, track_clicks: bool) -> Self {
            self.track_clicks = Some(track_clicks);
            self
        }

        /// The [`TransactionalEmail::updated_at`] you read: when the transactional email changed
        /// since, the update fails with 409 `conflict` instead of overwriting that change.
        #[inline]
        pub fn with_expected_updated_at(mut self, updated_at: &str) -> Self {
            self.expected_updated_at = Some(updated_at.to_owned());
            self
        }
    }

    /// One variable of the template version a transactional email sends.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TransactionalEmailVariable {
        pub key: String,
        /// `string` or `number`.
        #[serde(rename = "type")]
        pub variable_type: String,
        /// The value used when a send does not pass the variable (`None`: required).
        #[serde(default)]
        pub fallback_value: Option<Value>,
    }

    /// A transactional email.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TransactionalEmail {
        /// `tx_` and 27 letters and digits.
        pub id: TransactionalEmailId,
        pub name: String,
        /// The template whose published version is the content.
        pub template_id: TemplateId,
        #[serde(default)]
        pub template_name: Option<String>,
        #[serde(default)]
        pub template_alias: Option<String>,
        /// The template was deleted: sends fail until another template is chosen.
        #[serde(default)]
        pub template_deleted: bool,
        /// The template version sends use now (`None` when the template has no published
        /// version).
        #[serde(default)]
        pub published_version_id: Option<String>,
        #[serde(default)]
        pub published_version_number: Option<u32>,
        pub from: String,
        pub subject: String,
        #[serde(default)]
        pub preview_text: Option<String>,
        #[serde(default)]
        pub reply_to: Vec<String>,
        /// Open tracking (off by default).
        #[serde(default)]
        pub track_opens: bool,
        /// Click tracking (off by default).
        #[serde(default)]
        pub track_clicks: bool,
        /// `api` or `dashboard`.
        pub created_via: String,
        pub created_at: String,
        /// Pass it to [`UpdateTransactionalEmailOptions::with_expected_updated_at`].
        pub updated_at: String,
        /// The published version's variables (only from
        /// [`crate::services::TransactionalSvc::get`]): what a send's variables fill.
        #[serde(default)]
        pub variables: Option<Vec<TransactionalEmailVariable>>,
    }

    /// The answer to [`crate::services::TransactionalSvc::delete`].
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeleteTransactionalEmailResponse {
        pub id: TransactionalEmailId,
        pub deleted: bool,
    }

    /// The recipients and variables of [`crate::services::TransactionalSvc::send_test`].
    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    pub struct SendTestTransactionalEmailOptions {
        to: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        variables: Option<HashMap<String, Value>>,
    }

    impl SendTestTransactionalEmailOptions {
        /// Creates a new [`SendTestTransactionalEmailOptions`] for 1 to 5 addresses.
        pub fn new<T, A>(to: T) -> Self
        where
            T: IntoIterator<Item = A>,
            A: Into<String>,
        {
            Self {
                to: to.into_iter().map(Into::into).collect(),
                variables: None,
            }
        }

        /// Sets a template variable.
        #[inline]
        pub fn with_variable(mut self, key: &str, value: impl Into<Value>) -> Self {
            let variables = self.variables.get_or_insert_with(HashMap::new);
            let _previous = variables.insert(key.to_owned(), value.into());
            self
        }
    }

    /// Query parameters for [`crate::services::TransactionalSvc::list`].
    ///
    /// A [`ListOptions`] converts into this, for pagination without filters.
    ///
    /// ## Example
    ///
    /// ```
    /// # use sendio::types::ListTransactionalEmailsOptions;
    /// let list_opts = ListTransactionalEmailsOptions::new()
    ///   .with_search("sign-in")
    ///   .with_template_id("signin-code")
    ///   .with_limit(10);
    /// ```
    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    #[serde(bound(serialize = ""))]
    pub struct ListTransactionalEmailsOptions<List = TimeNotSpecified> {
        #[serde(skip_serializing_if = "Option::is_none")]
        search: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        template_id: Option<String>,

        #[serde(flatten)]
        pagination: ListOptions<List>,
    }

    impl Default for ListTransactionalEmailsOptions {
        fn default() -> Self {
            ListOptions::default().into()
        }
    }

    impl<T> From<ListOptions<T>> for ListTransactionalEmailsOptions<T> {
        fn from(pagination: ListOptions<T>) -> Self {
            Self {
                search: None,
                template_id: None,
                pagination,
            }
        }
    }

    impl ListTransactionalEmailsOptions<TimeNotSpecified> {
        /// Creates a new [`ListTransactionalEmailsOptions`] with no filters.
        pub fn new() -> Self {
            Self::default()
        }

        /// The id before which we'll retrieve the items. This id will *not* be included in the
        /// list.
        #[inline]
        pub fn list_before(self, id: &str) -> ListTransactionalEmailsOptions<ListBefore> {
            ListTransactionalEmailsOptions {
                search: self.search,
                template_id: self.template_id,
                pagination: self.pagination.list_before(id),
            }
        }

        /// The id after which we'll retrieve the items. This id will *not* be included in the
        /// list.
        #[inline]
        pub fn list_after(self, id: &str) -> ListTransactionalEmailsOptions<ListAfter> {
            ListTransactionalEmailsOptions {
                search: self.search,
                template_id: self.template_id,
                pagination: self.pagination.list_after(id),
            }
        }
    }

    impl<T> ListTransactionalEmailsOptions<T> {
        /// Number of transactional emails to retrieve.
        ///
        /// - min: 1
        /// - max: 100
        #[inline]
        pub fn with_limit(mut self, limit: u8) -> Self {
            self.pagination = self.pagination.with_limit(limit);
            self
        }

        /// Only transactional emails whose name or subject matches (case-insensitive).
        #[inline]
        pub fn with_search(mut self, search: &str) -> Self {
            self.search = Some(search.to_owned());
            self
        }

        /// Only the transactional emails of this template (id or alias).
        #[inline]
        pub fn with_template_id(mut self, template_id: &str) -> Self {
            self.template_id = Some(template_id.to_owned());
            self
        }
    }

    /// The period and grouping of [`crate::services::TransactionalSvc::stats`]. Every field
    /// is optional.
    #[must_use]
    #[derive(Debug, Default, Clone, Serialize)]
    pub struct TransactionalEmailStatsOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timezone: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        granularity: Option<String>,
    }

    impl TransactionalEmailStatsOptions {
        /// Creates a new [`TransactionalEmailStatsOptions`]: the last 30 days, daily, in UTC.
        pub fn new() -> Self {
            Self::default()
        }

        /// The start: a date (`2026-11-01`) or an ISO 8601 time, at most 90 days before the
        /// end.
        #[inline]
        pub fn with_from(mut self, from: &str) -> Self {
            self.from = Some(from.to_owned());
            self
        }

        /// The end: a date or an ISO 8601 time.
        #[inline]
        pub fn with_to(mut self, to: &str) -> Self {
            self.to = Some(to.to_owned());
            self
        }

        /// The IANA time zone of the days (default UTC).
        #[inline]
        pub fn with_timezone(mut self, timezone: &str) -> Self {
            self.timezone = Some(timezone.to_owned());
            self
        }

        /// One template version: an id, or a version number of the current template.
        #[inline]
        pub fn with_version(mut self, version: &str) -> Self {
            self.version = Some(version.to_owned());
            self
        }

        /// `daily` (the default) or `hourly`.
        #[inline]
        pub fn with_granularity(mut self, granularity: &str) -> Self {
            self.granularity = Some(granularity.to_owned());
            self
        }
    }

    /// The numbers of a transactional email's sends. `totals`, `daily`, `per_version` and
    /// `top_links` have the template stats' shapes.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TransactionalEmailStats {
        pub transactional_id: TransactionalEmailId,
        #[serde(default)]
        pub template_id: Option<String>,
        #[serde(default)]
        pub range: Option<Value>,
        #[serde(default)]
        pub version: Option<Value>,
        #[serde(default)]
        pub live_version: Option<Value>,
        #[serde(default)]
        pub totals: Option<Value>,
        #[serde(default)]
        pub daily: Vec<Value>,
        #[serde(default)]
        pub per_version: Vec<Value>,
        #[serde(default)]
        pub top_links: Vec<Value>,
    }
}

/// Requests made against a one-shot local HTTP server, so the exact method, path, query,
/// `Idempotency-Key` header and JSON body the client sends can be asserted.
#[cfg(test)]
#[cfg(not(feature = "blocking"))]
#[allow(clippy::indexing_slicing)]
mod mock_test {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::{Receiver, channel};

    use serde_json::{Value, json};

    use crate::{
        ConfigBuilder, Sendio,
        list_opts::ListOptions,
        types::{
            CreateBroadcastOptions, CreateEmailBaseOptions, CreateTransactionalEmailOptions,
            ListTransactionalEmailsOptions, SendTestTransactionalEmailOptions,
            TransactionalEmailStatsOptions, UpdateBroadcastOptions,
            UpdateTransactionalEmailOptions,
        },
    };

    const TX: &str = "tx_2fQk8Jv0Zr4mN7pB1sXyA9cDe3H";

    #[derive(Debug)]
    struct Captured {
        method: String,
        /// Path and query string.
        target: String,
        idempotency_key: Option<String>,
        body: Option<Value>,
    }

    /// Serves one request with `response` (as JSON, with `status`) and reports what was received.
    fn mock(status: &str, response: &Value) -> (Sendio, Receiver<Captured>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let response = response.to_string();
        let status = status.to_owned();
        let (tx, rx) = channel();

        let _handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));

            let mut request_line = String::new();
            let _n = reader.read_line(&mut request_line).expect("request line");
            let mut parts = request_line.split_whitespace();
            let method = parts.next().unwrap_or_default().to_owned();
            let target = parts.next().unwrap_or_default().to_owned();

            let mut content_length = 0;
            let mut idempotency_key = None;
            loop {
                let mut line = String::new();
                let _n = reader.read_line(&mut line).expect("header");
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some((name, value)) = line.split_once(':') {
                    if name.eq_ignore_ascii_case("content-length") {
                        content_length = value.trim().parse().expect("content-length");
                    } else if name.eq_ignore_ascii_case("idempotency-key") {
                        idempotency_key = Some(value.trim().to_owned());
                    }
                }
            }

            let mut raw = vec![0; content_length];
            reader.read_exact(&mut raw).expect("body");
            let body = (!raw.is_empty()).then(|| serde_json::from_slice(&raw).expect("json"));

            let mut stream = stream;
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            )
            .expect("write");

            tx.send(Captured {
                method,
                target,
                idempotency_key,
                body,
            })
            .expect("send");
        });

        let config = ConfigBuilder::new("mx_test")
            .base_url(format!("http://{addr}").parse().expect("url"))
            .build();

        (Sendio::with_config(config), rx)
    }

    fn transactional_email() -> Value {
        json!({
            "object": "transactional_email",
            "id": TX,
            "name": "Sign-in code",
            "template_id": "9c3e1f4a-1111-4222-8333-944455556666",
            "template_name": "Sign-in",
            "template_alias": "signin-code",
            "template_deleted": false,
            "published_version_id": "0d3e1f4a-1111-4222-8333-944455556666",
            "published_version_number": 3,
            "from": "Acme <security@acme.test>",
            "subject": "Your code is {{CODE}}",
            "preview_text": null,
            "reply_to": [],
            "track_opens": false,
            "track_clicks": false,
            "created_via": "api",
            "created_at": "2026-11-16T09:00:00.123456+00:00",
            "updated_at": "2026-11-16T09:00:00.123456+00:00",
            "variables": [{ "key": "CODE", "type": "string", "fallback_value": null }],
        })
    }

    #[tokio::test]
    async fn create_sends_the_fields_and_the_idempotency_key() {
        let (sendio, rx) = mock("200 OK", &transactional_email());

        let tx = sendio
            .transactional
            .create(
                CreateTransactionalEmailOptions::new(
                    "Sign-in code",
                    "signin-code",
                    "Acme <security@acme.test>",
                    "Your code is {{CODE}}",
                )
                .with_reply("help@acme.test")
                .with_track_opens(false)
                .with_idempotency_key("create-signin"),
            )
            .await
            .expect("create");

        assert_eq!(&*tx.id, TX);
        assert_eq!(tx.published_version_number, Some(3));
        assert_eq!(tx.template_alias.as_deref(), Some("signin-code"));
        assert!(!tx.track_opens);
        assert_eq!(tx.preview_text, None);
        let variables = tx.variables.expect("variables");
        assert_eq!(variables[0].key, "CODE");
        assert_eq!(variables[0].variable_type, "string");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, "/transactional");
        assert_eq!(req.idempotency_key.as_deref(), Some("create-signin"));
        assert_eq!(
            req.body,
            Some(json!({
                "name": "Sign-in code",
                "template_id": "signin-code",
                "from": "Acme <security@acme.test>",
                "subject": "Your code is {{CODE}}",
                "reply_to": ["help@acme.test"],
                "track_opens": false,
            }))
        );
    }

    #[tokio::test]
    async fn create_without_idempotency_key_sends_no_header() {
        let (sendio, rx) = mock("200 OK", &transactional_email());

        let _tx = sendio
            .transactional
            .create(CreateTransactionalEmailOptions::new(
                "n",
                "t",
                "a@acme.test",
                "s",
            ))
            .await
            .expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(req.idempotency_key, None);
    }

    #[tokio::test]
    async fn list_sends_the_filters_and_paging() {
        let (sendio, rx) = mock(
            "200 OK",
            &json!({ "object": "list", "has_more": true, "data": [transactional_email()] }),
        );

        let list = sendio
            .transactional
            .list(
                ListTransactionalEmailsOptions::new()
                    .with_search("sign in")
                    .with_template_id("signin-code")
                    .with_limit(10)
                    .list_after(TX),
            )
            .await
            .expect("list");
        assert!(list.has_more);
        assert_eq!(&*list.data[0].id, TX);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "GET");
        assert_eq!(
            req.target,
            format!("/transactional?search=sign+in&template_id=signin-code&limit=10&after={TX}")
        );
    }

    #[tokio::test]
    async fn list_takes_plain_list_options() {
        let (sendio, rx) = mock(
            "200 OK",
            &json!({ "object": "list", "has_more": false, "data": [] }),
        );

        let _list = sendio
            .transactional
            .list(ListOptions::default())
            .await
            .expect("list");

        let req = rx.recv().expect("request");
        assert_eq!(req.target, "/transactional");
    }

    #[tokio::test]
    async fn get_reads_the_transactional_email() {
        let (sendio, rx) = mock("200 OK", &transactional_email());

        let tx = sendio.transactional.get(TX).await.expect("get");
        assert_eq!(tx.subject, "Your code is {{CODE}}");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "GET");
        assert_eq!(req.target, format!("/transactional/{TX}"));
    }

    #[tokio::test]
    async fn update_sends_only_the_fields_set() {
        let (sendio, rx) = mock("200 OK", &transactional_email());

        let _tx = sendio
            .transactional
            .update(
                TX,
                UpdateTransactionalEmailOptions::new()
                    .with_subject("Your sign-in code")
                    .with_preview_text("")
                    .with_reply_to(&[])
                    .with_track_clicks(true)
                    .with_expected_updated_at("2026-11-16T09:00:00.123456+00:00"),
            )
            .await
            .expect("update");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "PATCH");
        assert_eq!(req.target, format!("/transactional/{TX}"));
        assert_eq!(
            req.body,
            Some(json!({
                "subject": "Your sign-in code",
                "preview_text": "",
                "reply_to": [],
                "track_clicks": true,
                "expected_updated_at": "2026-11-16T09:00:00.123456+00:00",
            }))
        );
    }

    #[tokio::test]
    async fn update_conflict_is_an_error() {
        let (sendio, _rx) = mock(
            "409 Conflict",
            &json!({ "name": "conflict", "message": "Someone else changed it.", "statusCode": 409 }),
        );

        let res = sendio
            .transactional
            .update(TX, UpdateTransactionalEmailOptions::new().with_name("n"))
            .await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn delete_reads_the_answer() {
        let (sendio, rx) = mock(
            "200 OK",
            &json!({ "object": "transactional_email", "id": TX, "deleted": true }),
        );

        let res = sendio.transactional.delete(TX).await.expect("delete");
        assert!(res.deleted);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "DELETE");
        assert_eq!(req.target, format!("/transactional/{TX}"));
    }

    #[tokio::test]
    async fn send_test_sends_the_recipients_and_variables() {
        let (sendio, rx) = mock("200 OK", &json!({ "id": "email_1" }));

        let res = sendio
            .transactional
            .send_test(
                TX,
                SendTestTransactionalEmailOptions::new(["a@acme.test"])
                    .with_variable("CODE", "123456"),
            )
            .await
            .expect("test");
        assert_eq!(&*res.id, "email_1");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, format!("/transactional/{TX}/test"));
        assert_eq!(
            req.body,
            Some(json!({ "to": ["a@acme.test"], "variables": { "CODE": "123456" } }))
        );
    }

    #[tokio::test]
    async fn stats_sends_the_period() {
        let (sendio, rx) = mock(
            "200 OK",
            &json!({
                "object": "transactional_email_stats",
                "transactional_id": TX,
                "template_id": "t",
                "totals": { "sent": 4 },
                "daily": [{ "date": "2026-11-16", "sent": 4 }],
                "per_version": [],
                "top_links": [],
            }),
        );

        let stats = sendio
            .transactional
            .stats(
                TX,
                TransactionalEmailStatsOptions::new()
                    .with_from("2026-11-01")
                    .with_to("2026-11-16")
                    .with_timezone("Europe/Paris")
                    .with_granularity("hourly"),
            )
            .await
            .expect("stats");
        assert_eq!(&*stats.transactional_id, TX);
        assert_eq!(stats.daily.len(), 1);
        assert_eq!(stats.totals, Some(json!({ "sent": 4 })));

        let req = rx.recv().expect("request");
        assert_eq!(
            req.target,
            format!(
                "/transactional/{TX}/stats?from=2026-11-01&to=2026-11-16&timezone=Europe%2FParis&granularity=hourly"
            )
        );
    }

    #[tokio::test]
    async fn email_send_with_a_transactional_id_leaves_from_and_subject_out() {
        let (sendio, rx) = mock("200 OK", &json!({ "id": "email_1" }));

        let email = CreateEmailBaseOptions::transactional(TX, ["ada@example.com"])
            .with_variable("CODE", "123456")
            .with_track_opens(false)
            .with_track_clicks(false);
        let _res = sendio.emails.send(email).await.expect("send");

        let req = rx.recv().expect("request");
        assert_eq!(req.target, "/emails");
        assert_eq!(
            req.body,
            Some(json!({
                "transactional_id": TX,
                "to": ["ada@example.com"],
                "variables": { "CODE": "123456" },
                "track_opens": false,
                "track_clicks": false,
            }))
        );
    }

    #[tokio::test]
    async fn email_get_reads_the_transactional_id() {
        let (sendio, _rx) = mock(
            "200 OK",
            &json!({
                "object": "email",
                "id": "email_1",
                "message_id": null,
                "from": "Acme <security@acme.test>",
                "to": ["ada@example.com"],
                "subject": "Your code is 123456",
                "created_at": "2026-11-16T09:00:00Z",
                "html": null,
                "text": null,
                "bcc": null,
                "cc": null,
                "reply_to": null,
                "last_event": "sent",
                "transactional_id": TX,
            }),
        );

        let email = sendio.emails.get("email_1").await.expect("get");
        assert_eq!(email.transactional_id.as_deref(), Some(TX));
    }

    #[tokio::test]
    async fn broadcast_create_sends_the_template_link() {
        let (sendio, rx) = mock("200 OK", &json!({ "object": "broadcast", "id": "b1" }));

        let _res = sendio
            .broadcasts
            .create(
                CreateBroadcastOptions::draft()
                    .with_name("Launch")
                    .with_segment_id("s1")
                    .with_template_id("launch")
                    .with_template_version_id("v1"),
            )
            .await
            .expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(
            req.body,
            Some(json!({
                "name": "Launch",
                "segment_id": "s1",
                "template_id": "launch",
                "template_version_id": "v1",
            }))
        );
    }

    #[tokio::test]
    async fn broadcast_update_detach_sends_template_id_null() {
        let (sendio, rx) = mock("200 OK", &json!({ "object": "broadcast", "id": "b1" }));

        let _res = sendio
            .broadcasts
            .update("b1", UpdateBroadcastOptions::new().detach_template())
            .await
            .expect("update");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "PATCH");
        assert_eq!(req.body, Some(json!({ "template_id": null })));
    }

    #[tokio::test]
    async fn broadcast_update_version_only() {
        let (sendio, rx) = mock("200 OK", &json!({ "object": "broadcast", "id": "b1" }));

        let _res = sendio
            .broadcasts
            .update(
                "b1",
                UpdateBroadcastOptions::new().with_template_version_id("v2"),
            )
            .await
            .expect("update");

        let req = rx.recv().expect("request");
        assert_eq!(req.body, Some(json!({ "template_version_id": "v2" })));
    }

    #[tokio::test]
    async fn broadcast_get_reads_the_template_link_and_unfillable_variables() {
        let (sendio, _rx) = mock(
            "200 OK",
            &json!({
                "object": "broadcast",
                "id": "b1",
                "name": "Launch",
                "status": "draft",
                "created_at": "2026-11-16T09:00:00Z",
                "scheduled_at": null,
                "sent_at": null,
                "from": null,
                "subject": null,
                "reply_to": null,
                "preview_text": null,
                "text": null,
                "html": null,
                "template_id": "t1",
                "template_version_id": "v1",
                "template_name": "Launch",
                "template_version_number": 2,
                "template_version_name": "Final",
                "template_deleted": false,
                "unfillable_variables": ["RESET_URL"],
            }),
        );

        let b = sendio.broadcasts.get("b1").await.expect("get");
        assert_eq!(b.template_id.as_deref(), Some("t1"));
        assert_eq!(b.template_version_id.as_deref(), Some("v1"));
        assert_eq!(b.template_name.as_deref(), Some("Launch"));
        assert_eq!(b.template_version_number, Some(2));
        assert_eq!(b.template_version_name.as_deref(), Some("Final"));
        assert_eq!(b.template_deleted, Some(false));
        assert_eq!(b.unfillable_variables, Some(vec!["RESET_URL".to_owned()]));
    }
}
