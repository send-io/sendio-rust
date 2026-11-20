use std::sync::Arc;

use reqwest::Method;

use crate::{
    Config, Result,
    list_opts::ListResponse,
    types::{
        CreateMailboxOptions, DeleteMailboxResponse, Mailbox, ResetMailboxPasswordResponse,
        UpdateMailboxOptions,
    },
};

/// `Sendio` APIs for `/mailboxes` endpoints.
///
/// Mailboxes are email accounts with an IMAP/SMTP login.
#[derive(Clone, Debug)]
pub struct MailboxesSvc(pub(crate) Arc<Config>);

impl MailboxesSvc {
    /// Retrieve every mailbox in the project. This endpoint is not paginated.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list(&self) -> Result<ListResponse<Mailbox>> {
        let request = self.0.build(Method::GET, "/mailboxes");
        let response = self.0.send(request).await?;
        let content = response.json::<ListResponse<Mailbox>>().await?;

        Ok(content)
    }

    /// Create a mailbox.
    ///
    /// The domain must be verified; the API answers `409` if the address already exists.
    /// When IMAP/SMTP is ready, the returned [`Mailbox`] carries its
    /// [`credentials`](Mailbox::credentials), which are only shown once.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn create(&self, mailbox: CreateMailboxOptions) -> Result<Mailbox> {
        let request = self.0.build(Method::POST, "/mailboxes");
        let response = self.0.send(request.json(&mailbox)).await?;
        let content = response.json::<Mailbox>().await?;

        Ok(content)
    }

    /// Turn warmup on or off for a mailbox.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn update(&self, mailbox_id: &str, update: UpdateMailboxOptions) -> Result<Mailbox> {
        let path = format!("/mailboxes/{mailbox_id}");

        let request = self.0.build(Method::PATCH, &path);
        let response = self.0.send(request.json(&update)).await?;
        let content = response.json::<Mailbox>().await?;

        Ok(content)
    }

    /// Set a new password for a mailbox and return the new credentials.
    ///
    /// The old password stops working. The password is also an API key for that mailbox.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn reset_password(&self, mailbox_id: &str) -> Result<ResetMailboxPasswordResponse> {
        let path = format!("/mailboxes/{mailbox_id}/password");

        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<ResetMailboxPasswordResponse>().await?;

        Ok(content)
    }

    /// Delete a mailbox. This removes the email account, and the agent inbox on it if there is
    /// one.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn delete(&self, mailbox_id: &str) -> Result<DeleteMailboxResponse> {
        let path = format!("/mailboxes/{mailbox_id}");

        let request = self.0.build(Method::DELETE, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<DeleteMailboxResponse>().await?;

        Ok(content)
    }
}

#[allow(unreachable_pub)]
pub mod types {
    use serde::{Deserialize, Serialize};

    use crate::types::DomainId;

    crate::define_id_type!(MailboxId);

    /// See [relevant docs].
    ///
    /// [relevant docs]: <https://developer.send.io/docs/api>
    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    pub struct CreateMailboxOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        address: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        local_part: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        domain_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        org: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
    }

    impl CreateMailboxOptions {
        const fn empty() -> Self {
            Self {
                address: None,
                local_part: None,
                domain_id: None,
                username: None,
                org: None,
                display_name: None,
            }
        }

        /// A mailbox for a full address (e.g. `sales@yourdomain.com`) on a verified domain.
        pub fn address(address: impl Into<String>) -> Self {
            Self {
                address: Some(address.into()),
                ..Self::empty()
            }
        }

        /// A mailbox for `local_part` on the verified domain `domain_id`.
        pub fn local_part(local_part: impl Into<String>, domain_id: impl Into<String>) -> Self {
            Self {
                local_part: Some(local_part.into()),
                domain_id: Some(domain_id.into()),
                ..Self::empty()
            }
        }

        /// A free mailbox, with no domain of your own.
        pub fn free(username: impl Into<String>) -> Self {
            Self {
                username: Some(username.into()),
                ..Self::empty()
            }
        }

        /// The organization of a free mailbox.
        #[inline]
        pub fn with_org(mut self, org: impl Into<String>) -> Self {
            self.org = Some(org.into());
            self
        }

        /// The display name of a free mailbox.
        #[inline]
        pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
            self.display_name = Some(display_name.into());
            self
        }
    }

    /// List of changes to apply to a [`Mailbox`].
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize)]
    pub struct UpdateMailboxOptions {
        warmup_enabled: bool,
    }

    impl UpdateMailboxOptions {
        /// Turns warmup on (`true`) or off (`false`). The value is always sent.
        pub const fn new(warmup_enabled: bool) -> Self {
            Self { warmup_enabled }
        }
    }

    /// Whether a mailbox is a plain mailbox or carries an agent inbox.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum MailboxKind {
        Normal,
        Agent,
        /// A value this version of the SDK does not know.
        #[serde(other)]
        Unknown,
    }

    /// State of a mailbox's IMAP/SMTP login.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum MailboxImapSmtp {
        Ready,
        Pending,
        Retrying,
        Unavailable,
        /// A value this version of the SDK does not know.
        #[serde(other)]
        Unknown,
    }

    /// Sending health of a mailbox over the last 30 days.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum MailboxHealth {
        NoData,
        Healthy,
        AtRisk,
        Poor,
        /// A value this version of the SDK does not know.
        #[serde(other)]
        Unknown,
    }

    /// Received Mailbox.
    ///
    /// `sent`, `delivered`, `bounced` and `complained` cover the last 30 days; `sent_total` is
    /// all time.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Mailbox {
        pub id: MailboxId,
        pub address: String,
        pub kind: MailboxKind,
        pub agent_inbox_id: Option<String>,
        pub imap_smtp: MailboxImapSmtp,
        pub domain_id: Option<DomainId>,
        pub domain: Option<String>,
        pub warmup_enabled: bool,
        pub warmup_changed_at: Option<String>,
        pub created_at: String,
        pub sent: u64,
        pub sent_total: u64,
        pub delivered: u64,
        pub bounced: u64,
        pub complained: u64,
        pub last_sent_at: Option<String>,
        pub health: MailboxHealth,
        /// Whether `POST /emails` accepts this address as From now (the API's own rule).
        #[serde(default)]
        pub can_send: bool,
        /// Why it cannot send, in a short line; `None` when it can.
        #[serde(default)]
        pub send_reason: Option<String>,
        /// The IMAP/SMTP login. Only returned by `create`, when IMAP/SMTP is ready, and shown
        /// once.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub credentials: Option<MailboxCredentials>,
    }

    /// IMAP/SMTP login of a mailbox. The password is also an API key for that mailbox.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct MailboxCredentials {
        pub username: String,
        pub password: String,
        pub imap: MailboxServer,
        pub smtp: MailboxServer,
    }

    /// An IMAP or SMTP server endpoint.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct MailboxServer {
        pub host: String,
        pub port: u16,
        /// `ssl` for IMAP, `starttls` for SMTP.
        pub security: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResetMailboxPasswordResponse {
        /// Unique identifier for the mailbox.
        pub id: MailboxId,
        pub address: String,
        /// The new login. The old password no longer works.
        pub credentials: MailboxCredentials,
        pub note: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeleteMailboxResponse {
        /// Unique identifier for the mailbox.
        pub id: MailboxId,
        /// Indicates whether the mailbox was deleted successfully.
        pub deleted: bool,
    }
}

/// Requests made against a one-shot local HTTP server, so the exact method, path and JSON body
/// the client sends can be asserted.
#[cfg(test)]
#[cfg(not(feature = "blocking"))]
#[allow(clippy::indexing_slicing)]
mod mock_test {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::{Receiver, channel};

    use serde_json::{Value, json};

    use super::types::{
        CreateMailboxOptions, MailboxHealth, MailboxImapSmtp, MailboxKind, UpdateMailboxOptions,
    };
    use crate::{ConfigBuilder, Sendio};

    const ID: &str = "8b0d5c1e-6a5f-4d0b-9a57-0f4c2f1e9d10";

    #[derive(Debug)]
    struct Captured {
        method: String,
        /// Path and query string.
        target: String,
        body: Option<Value>,
    }

    /// Serves one request with `response` (as JSON, status 200) and reports what was received.
    fn mock(response: &Value) -> (Sendio, Receiver<Captured>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let response = response.to_string();
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
            loop {
                let mut line = String::new();
                let _n = reader.read_line(&mut line).expect("header");
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    content_length = value.trim().parse().expect("content-length");
                }
            }

            let mut raw = vec![0; content_length];
            reader.read_exact(&mut raw).expect("body");
            let body = (!raw.is_empty()).then(|| serde_json::from_slice(&raw).expect("json"));

            let mut stream = stream;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            )
            .expect("write");

            tx.send(Captured {
                method,
                target,
                body,
            })
            .expect("send");
        });

        let config = ConfigBuilder::new("mx_test")
            .base_url(format!("http://{addr}").parse().expect("url"))
            .build();

        (Sendio::with_config(config), rx)
    }

    fn credentials() -> Value {
        json!({
            "username": "sales@example.com",
            "password": "mbx_secret",
            "imap": { "host": "imap.send.io", "port": 993, "security": "ssl" },
            "smtp": { "host": "smtp.send.io", "port": 587, "security": "starttls" },
        })
    }

    fn mailbox() -> Value {
        json!({
            "object": "mailbox",
            "id": ID,
            "address": "sales@example.com",
            "kind": "normal",
            "agent_inbox_id": null,
            "imap_smtp": "ready",
            "domain_id": "dom_1",
            "domain": "example.com",
            "warmup_enabled": false,
            "warmup_changed_at": null,
            "created_at": "2026-10-01T10:00:00Z",
            "sent": 12,
            "sent_total": 40,
            "delivered": 11,
            "bounced": 1,
            "complained": 0,
            "last_sent_at": "2026-10-20T09:00:00Z",
            "health": "healthy",
        })
    }

    #[tokio::test]
    async fn list_parses_mailboxes() {
        let free = json!({
            "object": "mailbox",
            "id": "mb_free",
            "address": "bob@acme.send.io",
            "kind": "agent",
            "agent_inbox_id": "inb_9",
            "imap_smtp": "pending",
            "domain_id": null,
            "domain": null,
            "warmup_enabled": true,
            "warmup_changed_at": "2026-10-02T10:00:00Z",
            "created_at": "2026-10-01T10:00:00Z",
            "sent": 0, "sent_total": 0, "delivered": 0, "bounced": 0, "complained": 0,
            "last_sent_at": null,
            "health": "no_data",
            "pool": "free",
            "origin": "signup",
            "receive_only": true,
            "sending": { "enabled": false },
        });
        let (sendio, rx) = mock(&json!({
            "object": "list",
            "has_more": false,
            "data": [mailbox(), free],
        }));

        let list = sendio.mailboxes.list().await.expect("list");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "GET");
        assert_eq!(req.target, "/mailboxes");
        assert_eq!(req.body, None);

        assert!(!list.has_more);
        assert_eq!(list.len(), 2);
        let mb = &list[0];
        assert_eq!(&*mb.id, ID);
        assert_eq!(mb.address, "sales@example.com");
        assert_eq!(mb.kind, MailboxKind::Normal);
        assert_eq!(mb.agent_inbox_id, None);
        assert_eq!(mb.imap_smtp, MailboxImapSmtp::Ready);
        assert_eq!(mb.domain_id.as_deref(), Some("dom_1"));
        assert_eq!(mb.domain.as_deref(), Some("example.com"));
        assert!(!mb.warmup_enabled);
        assert_eq!(mb.warmup_changed_at, None);
        assert_eq!(
            (
                mb.sent,
                mb.sent_total,
                mb.delivered,
                mb.bounced,
                mb.complained
            ),
            (12, 40, 11, 1, 0)
        );
        assert_eq!(mb.last_sent_at.as_deref(), Some("2026-10-20T09:00:00Z"));
        assert_eq!(mb.health, MailboxHealth::Healthy);
        assert_eq!(mb.credentials, None);

        let free = &list[1];
        assert_eq!(free.kind, MailboxKind::Agent);
        assert_eq!(free.agent_inbox_id.as_deref(), Some("inb_9"));
        assert_eq!(free.imap_smtp, MailboxImapSmtp::Pending);
        assert_eq!(free.domain_id, None);
        assert!(free.warmup_enabled);
        assert_eq!(free.health, MailboxHealth::NoData);
    }

    #[tokio::test]
    async fn create_sends_local_part_and_parses_credentials() {
        let mut response = mailbox();
        response["credentials"] = credentials();
        let (sendio, rx) = mock(&response);

        let mb = sendio
            .mailboxes
            .create(CreateMailboxOptions::local_part("sales", "dom_1"))
            .await
            .expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, "/mailboxes");
        assert_eq!(
            req.body,
            Some(json!({ "local_part": "sales", "domain_id": "dom_1" }))
        );

        let creds = mb.credentials.expect("credentials");
        assert_eq!(creds.username, "sales@example.com");
        assert_eq!(creds.password, "mbx_secret");
        assert_eq!(creds.imap.host, "imap.send.io");
        assert_eq!(creds.imap.port, 993);
        assert_eq!(creds.imap.security, "ssl");
        assert_eq!(creds.smtp.host, "smtp.send.io");
        assert_eq!(creds.smtp.port, 587);
        assert_eq!(creds.smtp.security, "starttls");
    }

    #[tokio::test]
    async fn create_sends_address() {
        let (sendio, rx) = mock(&mailbox());

        let _mb = sendio
            .mailboxes
            .create(CreateMailboxOptions::address("sales@example.com"))
            .await
            .expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(req.body, Some(json!({ "address": "sales@example.com" })));
    }

    #[tokio::test]
    async fn create_free_sends_username_org_and_display_name() {
        let (sendio, rx) = mock(&mailbox());

        let opts = CreateMailboxOptions::free("bob")
            .with_org("acme")
            .with_display_name("Bob");
        let _mb = sendio.mailboxes.create(opts).await.expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(
            req.body,
            Some(json!({ "username": "bob", "org": "acme", "display_name": "Bob" }))
        );
    }

    #[tokio::test]
    async fn update_sends_warmup_false() {
        let (sendio, rx) = mock(&mailbox());

        let mb = sendio
            .mailboxes
            .update(ID, UpdateMailboxOptions::new(false))
            .await
            .expect("update");
        assert!(!mb.warmup_enabled);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "PATCH");
        assert_eq!(req.target, format!("/mailboxes/{ID}"));
        assert_eq!(req.body, Some(json!({ "warmup_enabled": false })));
    }

    #[tokio::test]
    async fn update_sends_warmup_true() {
        let (sendio, rx) = mock(&mailbox());

        let _mb = sendio
            .mailboxes
            .update(ID, UpdateMailboxOptions::new(true))
            .await
            .expect("update");

        let req = rx.recv().expect("request");
        assert_eq!(req.body, Some(json!({ "warmup_enabled": true })));
    }

    #[tokio::test]
    async fn reset_password_posts_without_body() {
        let (sendio, rx) = mock(&json!({
            "object": "mailbox_credentials",
            "id": ID,
            "address": "sales@example.com",
            "credentials": credentials(),
            "note": "The old password no longer works.",
        }));

        let res = sendio
            .mailboxes
            .reset_password(ID)
            .await
            .expect("reset password");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, format!("/mailboxes/{ID}/password"));
        assert_eq!(req.body, None);

        assert_eq!(&*res.id, ID);
        assert_eq!(res.address, "sales@example.com");
        assert_eq!(res.credentials.password, "mbx_secret");
        assert_eq!(res.credentials.imap.port, 993);
        assert_eq!(res.credentials.smtp.security, "starttls");
        assert_eq!(res.note, "The old password no longer works.");
    }

    #[tokio::test]
    async fn delete_parses_response() {
        let (sendio, rx) = mock(&json!({ "object": "mailbox", "id": ID, "deleted": true }));

        let res = sendio.mailboxes.delete(ID).await.expect("delete");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "DELETE");
        assert_eq!(req.target, format!("/mailboxes/{ID}"));
        assert_eq!(req.body, None);

        assert_eq!(&*res.id, ID);
        assert!(res.deleted);
    }

    #[test]
    fn unknown_enum_values_deserialize() {
        let mut value = mailbox();
        value["kind"] = json!("shared");
        value["imap_smtp"] = json!("migrating");
        value["health"] = json!("excellent");

        let mb = serde_json::from_value::<super::types::Mailbox>(value).expect("parse");
        assert_eq!(mb.kind, MailboxKind::Unknown);
        assert_eq!(mb.imap_smtp, MailboxImapSmtp::Unknown);
        assert_eq!(mb.health, MailboxHealth::Unknown);
    }
}
