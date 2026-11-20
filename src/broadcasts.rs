use std::{fmt::Display, sync::Arc};

use reqwest::Method;
use types::{
    BroadcastEstimate, BroadcastStats, BroadcastVersion, EstimateBroadcastOptions,
    ListBroadcastsOptions, ListRecipientsOptions, TestBroadcastResponse, UpdateBroadcastOptions,
    UpdateBroadcastResponse, UpdateBroadcastVersionOptions,
};

use crate::{Config, Result, list_opts::ListResponse};
use crate::{
    list_opts::ListOptions,
    types::{
        Broadcast, BroadcastClickedLink, BroadcastRecipient, CancelBroadcastResponse,
        CreateBroadcastOptions, CreateBroadcastResponse, DuplicateBroadcastResponse,
        RemoveBroadcastResponse, SendBroadcastOptions, SendBroadcastResponse,
    },
};

/// `Sendio` APIs for `/broadcasts` endpoints.
#[derive(Clone, Debug)]
pub struct BroadcastsSvc(pub(crate) Arc<Config>);

impl BroadcastsSvc {
    /// Create a new broadcast to send to your audience.
    ///
    /// Without [`CreateBroadcastOptions::with_send`] this saves a draft, which may leave any
    /// field empty.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        broadcast: CreateBroadcastOptions,
    ) -> Result<CreateBroadcastResponse> {
        let request = self.0.build(Method::POST, "/broadcasts");
        let response = self.0.send(request.json(&broadcast)).await?;
        let content = response.json::<CreateBroadcastResponse>().await?;

        Ok(content)
    }

    /// Start sending broadcasts to your audience through the Sendio API.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn send(&self, broadcast: SendBroadcastOptions) -> Result<SendBroadcastResponse> {
        let path = format!("/broadcasts/{}/send", broadcast.broadcast_id);

        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request.json(&broadcast)).await?;
        let content = response.json::<SendBroadcastResponse>().await?;

        Ok(content)
    }

    /// Retrieve a list of broadcasts, newest first.
    ///
    /// Takes [`ListOptions`] for pagination only, or [`ListBroadcastsOptions`] to also filter
    /// by status, segment or name.
    ///
    /// - Default limit: no limit (return everything)
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list<T>(
        &self,
        list_opts: impl Into<ListBroadcastsOptions<T>>,
    ) -> Result<ListResponse<Broadcast>> {
        let list_opts = list_opts.into();
        let request = self.0.build(Method::GET, "/broadcasts").query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<ListResponse<Broadcast>>().await?;

        Ok(content)
    }

    /// Retrieve a single broadcast.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get(&self, broadcast_id: &str) -> Result<Broadcast> {
        let path = format!("/broadcasts/{broadcast_id}");

        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<Broadcast>().await?;

        Ok(content)
    }

    /// Cancel a broadcast. A scheduled broadcast goes back to draft; one that is sending stops.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn cancel(&self, broadcast_id: &str) -> Result<CancelBroadcastResponse> {
        let path = format!("/broadcasts/{broadcast_id}/cancel");

        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<CancelBroadcastResponse>().await?;

        Ok(content)
    }

    /// Duplicate a broadcast.
    ///
    /// Creates a new draft with the same content as the source, named after it with " (copy)" appended.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn duplicate(&self, broadcast_id: &str) -> Result<DuplicateBroadcastResponse> {
        let path = format!("/broadcasts/{broadcast_id}/duplicate");

        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<DuplicateBroadcastResponse>().await?;

        Ok(content)
    }

    /// Remove an existing broadcast.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn delete(&self, broadcast_id: &str) -> Result<bool> {
        let path = format!("/broadcasts/{broadcast_id}");

        let request = self.0.build(Method::DELETE, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<RemoveBroadcastResponse>().await?;

        Ok(content.deleted)
    }

    /// Retrieve the links clicked in a broadcast, ranked by total clicks.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn clicked_links<T>(
        &self,
        broadcast_id: &str,
        list_opts: ListOptions<T>,
    ) -> Result<ListResponse<BroadcastClickedLink>> {
        let path = format!("/broadcasts/{broadcast_id}/clicked-links");

        let request = self.0.build(Method::GET, &path).query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response
            .json::<ListResponse<BroadcastClickedLink>>()
            .await?;

        Ok(content)
    }

    /// Update a broadcast. Only the fields that are set are changed.
    ///
    /// Once a broadcast is sending or sent, only its name can be changed.
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        broadcast_id: &str,
        update: UpdateBroadcastOptions,
    ) -> Result<UpdateBroadcastResponse> {
        let path = format!("/broadcasts/{broadcast_id}");

        let request = self.0.build(Method::PATCH, &path);
        let response = self.0.send(request.json(&update)).await?;
        let content = response.json::<UpdateBroadcastResponse>().await?;

        Ok(content)
    }

    /// Retrieve the recipients of a broadcast for a given event type, such as who opened,
    /// clicked, or bounced.
    ///
    /// - Default limit: 20
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn recipients<T>(
        &self,
        broadcast_id: &str,
        list_opts: ListRecipientsOptions<T>,
    ) -> Result<ListResponse<BroadcastRecipient>> {
        let path = format!("/broadcasts/{broadcast_id}/recipients");

        let request = self.0.build(Method::GET, &path).query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<ListResponse<BroadcastRecipient>>().await?;

        Ok(content)
    }

    /// Estimate how many contacts an audience reaches, before anything is saved.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn estimate(&self, audience: EstimateBroadcastOptions) -> Result<BroadcastEstimate> {
        let request = self.0.build(Method::POST, "/broadcasts/estimate");
        let response = self.0.send(request.json(&audience)).await?;
        let content = response.json::<BroadcastEstimate>().await?;

        Ok(content)
    }

    /// Send a test of a broadcast to 1 to 5 addresses, personalized with a sample contact and
    /// with the subject prefixed `[Test]`.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn test(&self, broadcast_id: &str, to: &[&str]) -> Result<TestBroadcastResponse> {
        let path = format!("/broadcasts/{broadcast_id}/test");
        let body = serde_json::json!({ "to": to });

        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request.json(&body)).await?;
        let content = response.json::<TestBroadcastResponse>().await?;

        Ok(content)
    }

    /// Retrieve the delivery and engagement stats of a broadcast.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn stats(&self, broadcast_id: &str) -> Result<BroadcastStats> {
        let path = format!("/broadcasts/{broadcast_id}/stats");

        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<BroadcastStats>().await?;

        Ok(content)
    }

    /// Retrieve the saved versions of a broadcast, newest first.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list_versions<T>(
        &self,
        broadcast_id: &str,
        list_opts: ListOptions<T>,
    ) -> Result<ListResponse<BroadcastVersion>> {
        let path = format!("/broadcasts/{broadcast_id}/versions");

        let request = self.0.build(Method::GET, &path).query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<ListResponse<BroadcastVersion>>().await?;

        Ok(content)
    }

    /// Retrieve a version of a broadcast by its version number or id.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get_version(
        &self,
        broadcast_id: &str,
        version: impl Display,
    ) -> Result<BroadcastVersion> {
        let path = format!("/broadcasts/{broadcast_id}/versions/{version}");

        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<BroadcastVersion>().await?;

        Ok(content)
    }

    /// Label a version of a broadcast with a name and/or comment. Its content cannot change.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn update_version(
        &self,
        broadcast_id: &str,
        version: impl Display,
        update: UpdateBroadcastVersionOptions,
    ) -> Result<BroadcastVersion> {
        let path = format!("/broadcasts/{broadcast_id}/versions/{version}");

        let request = self.0.build(Method::PATCH, &path);
        let response = self.0.send(request.json(&update)).await?;
        let content = response.json::<BroadcastVersion>().await?;

        Ok(content)
    }

    /// Restore a version into a draft or scheduled broadcast. The result is saved as a new
    /// version, which is returned.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn restore_version(
        &self,
        broadcast_id: &str,
        version: impl Display,
    ) -> Result<BroadcastVersion> {
        let path = format!("/broadcasts/{broadcast_id}/versions/{version}/restore");

        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<BroadcastVersion>().await?;

        Ok(content)
    }
}

#[allow(unreachable_pub)]
pub mod types {
    use ecow::EcoString;
    use serde::{Deserialize, Serialize};

    use crate::{
        list_opts::{ListAfter, ListBefore, ListOptions, TimeNotSpecified},
        types::{ContactId, SegmentId, TopicId},
    };

    /// Who a broadcast goes to.
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
    #[must_use]
    #[serde(rename_all = "snake_case")]
    pub enum BroadcastAudience {
        /// The subscribed members of the broadcast's segments (the default).
        Segments,
        /// Every contact.
        All,
    }

    /// What the unsubscribe link of a broadcast unsubscribes from.
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
    #[must_use]
    #[serde(rename_all = "snake_case")]
    pub enum BroadcastUnsubscribeScope {
        /// Every broadcast (the default).
        All,
        /// The broadcast's segments only.
        List,
    }

    /// How merge tags in a broadcast are rendered.
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
    #[must_use]
    #[serde(rename_all = "snake_case")]
    pub enum BroadcastTemplateEngine {
        /// `{{{FIRST_NAME|fallback}}}` style merge tags (the default).
        Simple,
        /// Handlebars, with the contact as data.
        Handlebars,
    }

    /// The content, sender and audience fields shared by create and update.
    #[derive(Debug, Clone, Serialize, Default)]
    struct BroadcastContent {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        segment_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        segment_ids: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude_segment_ids: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        audience: Option<BroadcastAudience>,
        #[serde(skip_serializing_if = "Option::is_none")]
        unsubscribe_scope: Option<BroadcastUnsubscribeScope>,
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subject: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        html: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        preview_text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        topic_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tracking: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        template_engine: Option<BroadcastTemplateEngine>,
        /// `Some(None)` sends `template_id: null` (detach); `None` leaves it out.
        #[allow(clippy::option_option)]
        #[serde(skip_serializing_if = "Option::is_none")]
        template_id: Option<Option<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        template_version_id: Option<String>,
    }

    /// Builder methods for the fields shared by [`CreateBroadcastOptions`] and
    /// [`UpdateBroadcastOptions`].
    macro_rules! broadcast_content_setters {
        ($ty:ident) => {
            impl $ty {
                /// Appends `reply_to` address to the broadcast.
                #[inline]
                pub fn with_reply(mut self, to: &str) -> Self {
                    let reply_to = self.content.reply_to.get_or_insert_with(Vec::new);
                    reply_to.push(to.to_owned());
                    self
                }

                /// Appends multiple `reply_to` addresses to the broadcast.
                #[inline]
                pub fn with_reply_multiple(mut self, to: &[String]) -> Self {
                    let reply_to = self.content.reply_to.get_or_insert_with(Vec::new);
                    reply_to.extend_from_slice(to);
                    self
                }

                /// Adds or overwrites the HTML version of the message.
                #[inline]
                pub fn with_html(mut self, html: &str) -> Self {
                    self.content.html = Some(html.to_owned());
                    self
                }

                /// Adds or overwrites the plain text version of the message. An empty string
                /// opts out of the text part generated from the HTML.
                #[inline]
                pub fn with_text(mut self, text: &str) -> Self {
                    self.content.text = Some(text.to_owned());
                    self
                }

                /// Sets the broadcast name.
                #[inline]
                pub fn with_name(mut self, name: &str) -> Self {
                    self.content.name = Some(name.to_owned());
                    self
                }

                /// Sets the sender. To include a friendly name, use the format
                /// `"Your Name <sender@domain.com>"`.
                #[inline]
                pub fn with_from(mut self, from: &str) -> Self {
                    self.content.from = Some(from.to_owned());
                    self
                }

                /// Sets the subject.
                #[inline]
                pub fn with_subject(mut self, subject: &str) -> Self {
                    self.content.subject = Some(subject.to_owned());
                    self
                }

                /// Sets the preview text shown by email clients next to the subject.
                #[inline]
                pub fn with_preview_text(mut self, preview_text: &str) -> Self {
                    self.content.preview_text = Some(preview_text.to_owned());
                    self
                }

                /// Scopes the broadcast to a topic: contacts unsubscribed from it are skipped.
                #[inline]
                pub fn with_topic_id(mut self, topic_id: &str) -> Self {
                    self.content.topic_id = Some(topic_id.to_owned());
                    self
                }

                /// Sets the segment to send to.
                #[inline]
                pub fn with_segment_id(mut self, segment_id: &str) -> Self {
                    self.content.segment_id = Some(segment_id.to_owned());
                    self
                }

                /// Sets several segments to send to. A contact in more than one gets one email.
                #[inline]
                pub fn with_segment_ids(mut self, segment_ids: &[&str]) -> Self {
                    self.content.segment_ids =
                        Some(segment_ids.iter().map(|&id| id.to_owned()).collect());
                    self
                }

                /// Sets segments whose members never get the broadcast.
                #[inline]
                pub fn with_exclude_segment_ids(mut self, segment_ids: &[&str]) -> Self {
                    self.content.exclude_segment_ids =
                        Some(segment_ids.iter().map(|&id| id.to_owned()).collect());
                    self
                }

                /// Sets who the broadcast goes to: its segments, or every contact.
                #[inline]
                pub fn with_audience(mut self, audience: BroadcastAudience) -> Self {
                    self.content.audience = Some(audience);
                    self
                }

                /// Sets what the unsubscribe link unsubscribes from.
                #[inline]
                pub fn with_unsubscribe_scope(mut self, scope: BroadcastUnsubscribeScope) -> Self {
                    self.content.unsubscribe_scope = Some(scope);
                    self
                }

                /// Turns open and click tracking on or off (on by default).
                #[inline]
                pub fn with_tracking(mut self, tracking: bool) -> Self {
                    self.content.tracking = Some(tracking);
                    self
                }

                /// Sets how merge tags are rendered.
                #[inline]
                pub fn with_template_engine(mut self, engine: BroadcastTemplateEngine) -> Self {
                    self.content.template_engine = Some(engine);
                    self
                }

                /// Makes the broadcast from a published template (id or alias): it copies the
                /// version's content, and its subject, sender, preview text and reply-to where
                /// the broadcast has none. Not with html, text or a template engine; on update,
                /// those give the broadcast its own content and detach it from its template.
                #[inline]
                pub fn with_template_id(mut self, template_id: &str) -> Self {
                    self.content.template_id = Some(Some(template_id.to_owned()));
                    self
                }

                /// Picks the published template version to copy (default: the one published
                /// now). Alone on update, it switches to another version of the broadcast's
                /// template.
                #[inline]
                pub fn with_template_version_id(mut self, template_version_id: &str) -> Self {
                    self.content.template_version_id = Some(template_version_id.to_owned());
                    self
                }
            }
        };
    }

    /// Details of a new `Broadcast`.
    ///
    /// A draft needs nothing: start from [`CreateBroadcastOptions::draft`]. Sending (with
    /// [`CreateBroadcastOptions::with_send`]) needs a sender, a subject, an HTML or text body
    /// and an audience (a segment, or [`BroadcastAudience::All`]).
    #[must_use]
    #[derive(Debug, Clone, Serialize, Default)]
    pub struct CreateBroadcastOptions {
        #[serde(flatten)]
        content: BroadcastContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        send: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        scheduled_at: Option<String>,
    }

    impl CreateBroadcastOptions {
        /// Creates a new [`CreateBroadcastOptions`] for a segment.
        ///
        /// - `segment_id`: The ID of the segment you want to send to.
        /// - `from`: To include a friendly name, use the format `"Your Name <sender@domain.com>"`.
        /// - `subject`: Email subject.
        pub fn new(segment_id: &str, from: &str, subject: &str) -> Self {
            Self::draft()
                .with_segment_id(segment_id)
                .with_from(from)
                .with_subject(subject)
        }

        /// Creates an empty [`CreateBroadcastOptions`]: a draft with no field set.
        pub fn draft() -> Self {
            Self::default()
        }

        /// When set to `true`, the broadcast will be sent or scheduled (if `scheduled_at` is
        /// provided) without requiring a separate call to the
        /// [`crate::broadcasts::BroadcastsSvc::send`] endpoint.
        #[inline]
        pub fn with_send(mut self, send: bool) -> Self {
            self.send = Some(send);
            self
        }

        /// Schedule email to be sent later. The date should be in language natural (e.g.: in 1 min)
        /// or ISO 8601 format (e.g: 2024-08-05T11:52:01.858Z). Requires `send: true`.
        #[inline]
        pub fn with_scheduled_at(mut self, scheduled_at: &str) -> Self {
            self.scheduled_at = Some(scheduled_at.to_owned());
            self
        }
    }

    broadcast_content_setters!(CreateBroadcastOptions);

    /// Changes to a `Broadcast`. Only the fields that are set are sent.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Default)]
    pub struct UpdateBroadcastOptions {
        #[serde(flatten)]
        content: BroadcastContent,
    }

    impl UpdateBroadcastOptions {
        /// Creates a new [`UpdateBroadcastOptions`].
        pub fn new() -> Self {
            Self::default()
        }

        /// Detaches the broadcast from its template (`template_id: null`); its content stays.
        #[inline]
        pub fn detach_template(mut self) -> Self {
            self.content.template_id = Some(None);
            self
        }
    }

    broadcast_content_setters!(UpdateBroadcastOptions);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateBroadcastResponse {
        /// Unique identifier for the updated broadcast.
        pub id: BroadcastId,
    }

    crate::define_id_type!(BroadcastId);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateBroadcastResponse {
        /// The ID of the created broadcast.
        pub id: BroadcastId,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    pub struct SendBroadcastOptions {
        /// Goes in the path only, never in the body.
        #[serde(skip)]
        pub(crate) broadcast_id: BroadcastId,

        #[serde(skip_serializing_if = "Option::is_none")]
        scheduled_at: Option<String>,
    }

    impl SendBroadcastOptions {
        pub fn new(broadcast_id: &str) -> Self {
            let broadcast_id = BroadcastId(EcoString::from(broadcast_id.to_owned()));

            Self {
                broadcast_id,
                scheduled_at: None,
            }
        }

        /// Schedule email to be sent later. The date should be in language natural (e.g.: in 1 min)
        /// or ISO 8601 format (e.g: 2024-08-05T11:52:01.858Z).
        #[inline]
        pub fn with_scheduled_at(mut self, scheduled_at: &str) -> Self {
            self.scheduled_at = Some(scheduled_at.to_owned());
            self
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SendBroadcastResponse {
        /// The ID of the sent broadcast.
        pub id: BroadcastId,
    }

    /// A broadcast.
    ///
    /// [`crate::broadcasts::BroadcastsSvc::get`] returns every field;
    /// [`crate::broadcasts::BroadcastsSvc::list`] returns the id, name, audience fields,
    /// status and dates.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Broadcast {
        pub id: BroadcastId,
        /// `None` for an unnamed draft.
        pub name: Option<String>,
        /// Deprecated alias of [`Broadcast::segment_id`].
        #[serde(default)]
        pub audience_id: Option<SegmentId>,
        /// The first segment; `None` for a draft without one or an all-contacts broadcast.
        #[serde(default)]
        pub segment_id: Option<SegmentId>,
        #[serde(default)]
        pub segment_ids: Option<Vec<SegmentId>>,
        #[serde(default)]
        pub exclude_segment_ids: Option<Vec<SegmentId>>,
        #[serde(default)]
        pub audience: Option<BroadcastAudience>,
        #[serde(default)]
        pub unsubscribe_scope: Option<BroadcastUnsubscribeScope>,
        pub status: String,
        pub created_at: String,
        pub scheduled_at: Option<String>,
        pub sent_at: Option<String>,
        pub from: Option<String>,
        pub subject: Option<String>,
        pub reply_to: Option<Vec<String>>,
        pub preview_text: Option<String>,
        pub text: Option<String>,
        pub html: Option<String>,
        #[serde(default)]
        pub topic_id: Option<TopicId>,
        #[serde(default)]
        pub tracking: Option<bool>,
        #[serde(default)]
        pub template_engine: Option<BroadcastTemplateEngine>,
        /// The number of the broadcast's current version.
        #[serde(default)]
        pub version_number: Option<u32>,
        #[serde(default)]
        pub failed_reason: Option<String>,
        #[serde(default)]
        pub updated_at: Option<String>,
        /// The template the broadcast was made from (`None` for its own content).
        #[serde(default)]
        pub template_id: Option<String>,
        /// The exact published template version it copied.
        #[serde(default)]
        pub template_version_id: Option<String>,
        #[serde(default)]
        pub template_name: Option<String>,
        #[serde(default)]
        pub template_version_number: Option<u32>,
        #[serde(default)]
        pub template_version_name: Option<String>,
        /// The template was deleted (the broadcast keeps its copy).
        #[serde(default)]
        pub template_deleted: Option<bool>,
        /// The template variables the broadcast has no value for (only from
        /// [`crate::broadcasts::BroadcastsSvc::get`], for a broadcast made from a template).
        /// While it is not empty, sending or scheduling is refused with 422
        /// `unfillable_variables`.
        #[serde(default)]
        pub unfillable_variables: Option<Vec<String>>,
    }

    /// Query parameters for [`crate::broadcasts::BroadcastsSvc::list`].
    ///
    /// A [`ListOptions`] converts into this, for pagination without filters.
    ///
    /// ## Example
    ///
    /// ```
    /// # use sendio::types::ListBroadcastsOptions;
    /// let list_opts = ListBroadcastsOptions::new()
    ///   .with_status("draft")
    ///   .with_search("launch")
    ///   .with_limit(10);
    /// ```
    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    #[serde(bound(serialize = ""))]
    pub struct ListBroadcastsOptions<List = TimeNotSpecified> {
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        segment_id: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        search: Option<String>,

        #[serde(flatten)]
        pagination: ListOptions<List>,
    }

    impl Default for ListBroadcastsOptions {
        fn default() -> Self {
            ListOptions::default().into()
        }
    }

    impl<T> From<ListOptions<T>> for ListBroadcastsOptions<T> {
        fn from(pagination: ListOptions<T>) -> Self {
            Self {
                status: None,
                segment_id: None,
                search: None,
                pagination,
            }
        }
    }

    impl ListBroadcastsOptions<TimeNotSpecified> {
        /// Creates a new [`ListBroadcastsOptions`] with no filters.
        pub fn new() -> Self {
            Self::default()
        }

        /// The id before which we'll retrieve the items. This id will *not* be included in the
        /// list.
        #[inline]
        pub fn list_before(self, id: &str) -> ListBroadcastsOptions<ListBefore> {
            ListBroadcastsOptions {
                status: self.status,
                segment_id: self.segment_id,
                search: self.search,
                pagination: self.pagination.list_before(id),
            }
        }

        /// The id after which we'll retrieve the items. This id will *not* be included in the
        /// list.
        #[inline]
        pub fn list_after(self, id: &str) -> ListBroadcastsOptions<ListAfter> {
            ListBroadcastsOptions {
                status: self.status,
                segment_id: self.segment_id,
                search: self.search,
                pagination: self.pagination.list_after(id),
            }
        }
    }

    impl<T> ListBroadcastsOptions<T> {
        /// Number of broadcasts to retrieve.
        ///
        /// - min: 1
        /// - max: 100
        #[inline]
        pub fn with_limit(mut self, limit: u8) -> Self {
            self.pagination = self.pagination.with_limit(limit);
            self
        }

        /// Only broadcasts with this status (`draft`, `scheduled`, `queued`, `sent`, ...).
        #[inline]
        pub fn with_status(mut self, status: &str) -> Self {
            self.status = Some(status.to_owned());
            self
        }

        /// Only broadcasts sent to this segment.
        #[inline]
        pub fn with_segment_id(mut self, segment_id: &str) -> Self {
            self.segment_id = Some(segment_id.to_owned());
            self
        }

        /// Only broadcasts whose name matches.
        #[inline]
        pub fn with_search(mut self, search: &str) -> Self {
            self.search = Some(search.to_owned());
            self
        }
    }

    /// The audience to estimate with [`crate::broadcasts::BroadcastsSvc::estimate`].
    #[must_use]
    #[derive(Debug, Clone, Serialize, Default)]
    pub struct EstimateBroadcastOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        segment_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        segment_ids: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude_segment_ids: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        audience: Option<BroadcastAudience>,
        #[serde(skip_serializing_if = "Option::is_none")]
        topic_id: Option<String>,
    }

    impl EstimateBroadcastOptions {
        /// Creates a new, empty [`EstimateBroadcastOptions`].
        pub fn new() -> Self {
            Self::default()
        }

        /// Sets the segment to send to.
        #[inline]
        pub fn with_segment_id(mut self, segment_id: &str) -> Self {
            self.segment_id = Some(segment_id.to_owned());
            self
        }

        /// Sets several segments to send to.
        #[inline]
        pub fn with_segment_ids(mut self, segment_ids: &[&str]) -> Self {
            self.segment_ids = Some(segment_ids.iter().map(|&id| id.to_owned()).collect());
            self
        }

        /// Sets segments whose members are left out.
        #[inline]
        pub fn with_exclude_segment_ids(mut self, segment_ids: &[&str]) -> Self {
            self.exclude_segment_ids = Some(segment_ids.iter().map(|&id| id.to_owned()).collect());
            self
        }

        /// Sets who the broadcast goes to: its segments, or every contact.
        #[inline]
        pub fn with_audience(mut self, audience: BroadcastAudience) -> Self {
            self.audience = Some(audience);
            self
        }

        /// Leaves out contacts unsubscribed from this topic.
        #[inline]
        pub fn with_topic_id(mut self, topic_id: &str) -> Self {
            self.topic_id = Some(topic_id.to_owned());
            self
        }
    }

    /// Contacts an audience skips.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
    pub struct BroadcastSkipped {
        /// Contacts whose email verification found them invalid.
        #[serde(default)]
        pub invalid: u64,
    }

    /// How many contacts an audience reaches.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct BroadcastEstimate {
        pub recipients: u64,
        #[serde(default)]
        pub skipped: BroadcastSkipped,
    }

    /// The result of [`crate::broadcasts::BroadcastsSvc::test`].
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct TestBroadcastResponse {
        /// How many test emails were sent.
        pub sent: u32,
    }

    /// How far a broadcast has been queued.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
    pub struct BroadcastProgress {
        #[serde(default)]
        pub queued: u64,
        #[serde(default)]
        pub total: u64,
    }

    /// Delivery and engagement stats of a broadcast. Rates are fractions between 0 and 1.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[serde(default)]
    pub struct BroadcastStats {
        pub status: String,
        pub recipients: u64,
        pub sent: u64,
        pub delivered: u64,
        pub bounced: u64,
        pub complained: u64,
        pub suppressed: u64,
        pub failed: u64,
        pub opened: u64,
        pub unique_opened: u64,
        pub clicked: u64,
        pub unique_clicked: u64,
        pub unsubscribed: u64,
        pub skipped: BroadcastSkipped,
        pub delivery_rate: f64,
        pub open_rate: f64,
        pub click_rate: f64,
        pub bounce_rate: f64,
        pub unsubscribe_rate: f64,
        pub progress: BroadcastProgress,
    }

    /// A saved version of a broadcast.
    ///
    /// [`crate::broadcasts::BroadcastsSvc::list_versions`] returns the summary fields; a single
    /// version also has its content, sender and audience fields.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BroadcastVersion {
        pub id: String,
        pub broadcast_id: BroadcastId,
        pub version_number: u32,
        pub name: Option<String>,
        pub comment: Option<String>,
        pub broadcast_name: Option<String>,
        pub subject: Option<String>,
        /// The version this one was restored from.
        pub restored_from: Option<u32>,
        pub created_at: String,
        /// Whether this is the broadcast's current version.
        #[serde(default)]
        pub current: bool,
        #[serde(default)]
        pub from: Option<String>,
        #[serde(default)]
        pub reply_to: Option<Vec<String>>,
        #[serde(default)]
        pub preview_text: Option<String>,
        #[serde(default)]
        pub html: Option<String>,
        #[serde(default)]
        pub text: Option<String>,
        #[serde(default)]
        pub template_engine: Option<BroadcastTemplateEngine>,
        #[serde(default)]
        pub tracking: Option<bool>,
        #[serde(default)]
        pub topic_id: Option<TopicId>,
        #[serde(default)]
        pub segment_ids: Option<Vec<SegmentId>>,
        #[serde(default)]
        pub exclude_segment_ids: Option<Vec<SegmentId>>,
        #[serde(default)]
        pub audience: Option<BroadcastAudience>,
        #[serde(default)]
        pub unsubscribe_scope: Option<BroadcastUnsubscribeScope>,
    }

    /// The label of a broadcast version.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Default)]
    pub struct UpdateBroadcastVersionOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        comment: Option<String>,
    }

    impl UpdateBroadcastVersionOptions {
        /// Creates a new, empty [`UpdateBroadcastVersionOptions`].
        pub fn new() -> Self {
            Self::default()
        }

        /// Sets the version's name.
        #[inline]
        pub fn with_name(mut self, name: &str) -> Self {
            self.name = Some(name.to_owned());
            self
        }

        /// Sets the version's comment.
        #[inline]
        pub fn with_comment(mut self, comment: &str) -> Self {
            self.comment = Some(comment.to_owned());
            self
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CancelBroadcastResponse {
        pub id: BroadcastId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DuplicateBroadcastResponse {
        /// Unique identifier for the duplicated broadcast.
        pub id: BroadcastId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RemoveBroadcastResponse {
        /// The ID of the broadcast.
        #[allow(dead_code)]
        pub id: BroadcastId,
        /// The deleted attribute indicates that the corresponding broadcast has been deleted.
        pub deleted: bool,
    }

    /// The recipient event type to filter by when listing [`BroadcastRecipient`]s.
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
    #[must_use]
    #[serde(rename_all = "snake_case")]
    pub enum BroadcastRecipientEventType {
        Sent,
        Delivered,
        Opened,
        Clicked,
        Bounced,
        Complained,
        Unsubscribed,
        Suppressed,
    }

    /// The classification of a bounce for a [`BroadcastRecipient`].
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
    #[must_use]
    #[serde(rename_all = "snake_case")]
    pub enum BroadcastRecipientBounceType {
        Permanent,
        Transient,
        Undetermined,
    }

    /// Query parameters for [`crate::broadcasts::BroadcastsSvc::recipients`].
    ///
    /// `before` and `after` are mutually exclusive; use [`ListRecipientsOptions::list_before`] or
    /// [`ListRecipientsOptions::list_after`] to pick one, same as [`ListOptions`].
    ///
    /// <https://developer.send.io/docs/api>
    ///
    /// ## Example
    ///
    /// ```
    /// # use sendio::types::{BroadcastRecipientEventType, ListRecipientsOptions};
    /// let list_opts =
    ///   ListRecipientsOptions::new(BroadcastRecipientEventType::Clicked).with_limit(10);
    /// ```
    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    // The `List` parameter only ever reaches the wire through `pagination`'s own (bound-free)
    // `Serialize` impl, so it never needs to implement `Serialize` itself; the phantom marker
    // types (e.g. `TimeNotSpecified`) intentionally don't.
    #[serde(bound(serialize = ""))]
    pub struct ListRecipientsOptions<List = TimeNotSpecified> {
        #[serde(rename = "type")]
        r#type: BroadcastRecipientEventType,

        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        bounce_type: Option<BroadcastRecipientBounceType>,

        #[serde(flatten)]
        pagination: ListOptions<List>,
    }

    impl ListRecipientsOptions<TimeNotSpecified> {
        /// Creates a new [`ListRecipientsOptions`], filtering recipients by the given event
        /// `type`.
        pub fn new(event_type: BroadcastRecipientEventType) -> Self {
            Self {
                r#type: event_type,
                email: None,
                bounce_type: None,
                pagination: ListOptions::default(),
            }
        }

        /// The id before which we'll retrieve the items. This id will *not* be included in the
        /// list.
        #[inline]
        pub fn list_before(self, id: &str) -> ListRecipientsOptions<ListBefore> {
            ListRecipientsOptions {
                r#type: self.r#type,
                email: self.email,
                bounce_type: self.bounce_type,
                pagination: self.pagination.list_before(id),
            }
        }

        /// The id after which we'll retrieve the items. This id will *not* be included in the
        /// list.
        #[inline]
        pub fn list_after(self, id: &str) -> ListRecipientsOptions<ListAfter> {
            ListRecipientsOptions {
                r#type: self.r#type,
                email: self.email,
                bounce_type: self.bounce_type,
                pagination: self.pagination.list_after(id),
            }
        }
    }

    impl<T> ListRecipientsOptions<T> {
        /// Number of recipients to retrieve.
        ///
        /// - min: 1
        /// - max: 100
        /// - default: 20
        #[inline]
        pub fn with_limit(mut self, limit: u8) -> Self {
            self.pagination = self.pagination.with_limit(limit);
            self
        }

        /// Filters recipients whose email contains this value.
        #[inline]
        pub fn with_email(mut self, email: &str) -> Self {
            self.email = Some(email.to_owned());
            self
        }

        /// Filters bounced recipients by bounce type.
        ///
        /// Only meaningful when `type` is [`BroadcastRecipientEventType::Bounced`].
        #[inline]
        pub fn with_bounce_type(mut self, bounce_type: BroadcastRecipientBounceType) -> Self {
            self.bounce_type = Some(bounce_type);
            self
        }
    }

    /// A link clicked by a [`BroadcastRecipient`]. Only present when `type` is `clicked`.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BroadcastRecipientClickedLink {
        pub url: String,
        pub clicks: u32,
    }

    /// A single recipient of a broadcast, matching the requested event `type`.
    ///
    /// <https://developer.send.io/docs/api>
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BroadcastRecipient {
        /// Opaque cursor identifying this row, used only for pagination. This does not identify
        /// any entity in Sendio; use [`BroadcastRecipient::contact_id`] to reference the contact.
        pub id: String,
        /// The ID of the contact associated with this recipient. `None` if the recipient's email
        /// no longer maps to a contact.
        pub contact_id: Option<ContactId>,
        pub email: String,
        /// The number of times this recipient triggered the event. Only present when the
        /// requested `type` is `opened` or `clicked`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub count: Option<u32>,
        /// The type of bounce. Only present when the requested `type` is `bounced`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub bounce_type: Option<BroadcastRecipientBounceType>,
        /// The links this recipient clicked. Only present when the requested `type` is `clicked`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub clicked_links: Option<Vec<BroadcastRecipientClickedLink>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BroadcastClickedLink {
        /// An opaque cursor for this row, used only for pagination. It does not identify any
        /// entity in Sendio.
        pub id: String,
        /// The URL that was clicked.
        pub url: String,
        /// Total number of clicks on this URL.
        pub clicks: u64,
        /// Number of unique clicks on this URL.
        pub unique_clicks: u64,
    }
}

#[cfg(test)]
#[allow(clippy::needless_return, clippy::indexing_slicing)]
mod test {
    #[cfg(not(feature = "blocking"))]
    use crate::{
        list_opts::ListOptions,
        test::{CLIENT, DebugResult},
        types::{
            CreateBroadcastOptions, CreateContactOptions, SendBroadcastOptions,
            UpdateBroadcastOptions,
        },
    };

    use super::types::{
        Broadcast, BroadcastClickedLink, BroadcastRecipient, BroadcastRecipientEventType,
        CancelBroadcastResponse, ListRecipientsOptions,
    };

    #[tokio_shared_rt::test(shared = true)]
    #[serial_test::serial]
    #[cfg(not(feature = "blocking"))]
    #[ignore = "needs SENDIO_API_KEY and a live API"]
    async fn create_send_broadcast() -> DebugResult<()> {
        let sendio = &*CLIENT;
        std::thread::sleep(std::time::Duration::from_secs(1));

        let audience_id = sendio.segments.create("audience").await?.id;

        let contact = CreateContactOptions::new("steve.wozniak@gmail.com")
            .with_first_name("Steve")
            .with_last_name("Wozniak")
            .with_unsubscribed(false)
            .with_audience_id(&audience_id);

        let _contact_id = sendio.contacts.create(contact).await?;

        let from = "Acme <onboarding@yourdomain.com>";
        let subject = "hello world";
        let html =
            "Hi {{{FIRST_NAME|there}}}, you can unsubscribe here: {{{SENDIO_UNSUBSCRIBE_URL}}}";

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Create
        let broadcast = CreateBroadcastOptions::new(&audience_id, from, subject).with_html(html);
        let res = sendio.broadcasts.create(broadcast).await?;

        std::thread::sleep(std::time::Duration::from_secs(4));

        // Send
        let opts = SendBroadcastOptions::new(&res.id);
        let _res = sendio.broadcasts.send(opts).await?;

        // Cleanup
        std::thread::sleep(std::time::Duration::from_secs(2));

        let deleted = sendio.segments.delete(&audience_id).await?;
        std::thread::sleep(std::time::Duration::from_secs(1));

        assert!(deleted);

        Ok(())
    }

    #[tokio_shared_rt::test(shared = true)]
    #[serial_test::serial]
    #[cfg(not(feature = "blocking"))]
    #[ignore = "needs SENDIO_API_KEY and a live API"]
    async fn list_get_broadcast() -> DebugResult<()> {
        let sendio = &*CLIENT;
        std::thread::sleep(std::time::Duration::from_secs(1));

        let broadcasts = sendio.broadcasts.list(ListOptions::default()).await?;
        assert!(!broadcasts.data.is_empty(), "No broadcasts found");
        let broadcast = broadcasts[0].clone();

        let _res = sendio.broadcasts.get(&broadcast.id.clone()).await?;
        let _deleted = sendio.broadcasts.delete(&broadcast.id).await;
        // TODO: This does not seem to be the case anymore?
        // Already used broadcasts cant be deleted
        // assert!(deleted.is_err());

        // Create fresh broadcast and delete that instead
        let audience_id = sendio.segments.create("audience").await?.id;
        let from = "Acme <onboarding@yourdomain.com>";
        let subject = "hello world";
        let text = "text";

        let broadcast = CreateBroadcastOptions::new(&audience_id, from, subject).with_text(text);
        let res = sendio.broadcasts.create(broadcast).await?;
        std::thread::sleep(std::time::Duration::from_secs(2));
        let deleted_broadcast = sendio.broadcasts.delete(&res.id).await;
        let deleted_audience = sendio.segments.delete(&audience_id).await;
        std::thread::sleep(std::time::Duration::from_secs(1));

        assert!(deleted_broadcast.is_ok());
        assert!(deleted_audience.is_ok());

        Ok(())
    }

    #[tokio_shared_rt::test(shared = true)]
    #[serial_test::serial]
    #[cfg(not(feature = "blocking"))]
    #[ignore = "needs SENDIO_API_KEY and a live API"]
    async fn update_broadcast() -> DebugResult<()> {
        let sendio = &*CLIENT;
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Create audience & broadcast
        let audience_id = sendio.segments.create("audience").await?.id;
        let from = "Acme <onboarding@yourdomain.com>";
        let subject = "hello world";

        let create_broadcast =
            CreateBroadcastOptions::new(&audience_id, from, subject).with_text("text");
        let broadcast_id = sendio.broadcasts.create(create_broadcast).await?.id;
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Assert subject == initial subject
        let broadcast = sendio.broadcasts.get(&broadcast_id).await?;
        assert_eq!(Some(subject.to_string()), broadcast.subject);

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Update subject
        let subject = "updated";
        let opts = UpdateBroadcastOptions::new().with_subject(subject);
        let _unused = sendio.broadcasts.update(&broadcast_id, opts).await?;

        // Assert subject == updated subject
        let broadcast = sendio.broadcasts.get(&broadcast_id).await?;
        assert_eq!(Some(subject.to_string()), broadcast.subject);

        let duplicate = sendio.broadcasts.duplicate(&broadcast_id).await?;
        assert_ne!(duplicate.id, broadcast_id);
        let deleted = sendio.broadcasts.delete(&duplicate.id).await?;
        assert!(deleted);

        // Delete
        let deleted = sendio.broadcasts.delete(&broadcast_id).await?;
        assert!(deleted);

        Ok(())
    }

    #[test]
    fn parse_broadcast_test() {
        let data = r#"{
    "object": "broadcast",
    "id": "498ee8e4-7aa2-4eb5-9f04-4194848049d1",
    "name": "Untitled",
    "audience_id": "fd644f07-a05a-467e-9bae-23bb7c35766a",
    "from": "Acme <onboarding@yourdomain.com>",
    "subject": "Hello!",
    "reply_to": [],
    "preview_text": null,
    "status": "scheduled",
    "created_at": "2024-12-18T18:05:09.905933+00:00",
    "scheduled_at": null,
    "sent_at": null
}"#;

        let _parsed = serde_json::from_str::<Broadcast>(data).expect("Parsing failed");
    }

    #[test]
    fn parse_unnamed_all_contacts_broadcast_test() {
        let data = r#"{
    "object": "broadcast",
    "id": "498ee8e4-7aa2-4eb5-9f04-4194848049d1",
    "name": null,
    "audience_id": null,
    "segment_id": null,
    "segment_ids": [],
    "exclude_segment_ids": ["fd644f07-a05a-467e-9bae-23bb7c35766a"],
    "audience": "all",
    "unsubscribe_scope": "list",
    "from": null,
    "subject": null,
    "reply_to": null,
    "preview_text": "Preview",
    "html": null,
    "text": null,
    "status": "draft",
    "created_at": "2024-12-18T18:05:09.905933+00:00",
    "scheduled_at": null,
    "sent_at": null,
    "topic_id": null,
    "design": null,
    "tracking": false,
    "template_engine": "handlebars",
    "failed_reason": null,
    "updated_at": "2024-12-18T18:05:09.905933+00:00",
    "version_number": 1
}"#;

        let parsed = serde_json::from_str::<Broadcast>(data).expect("Parsing failed");

        assert!(parsed.name.is_none());
        assert!(parsed.audience_id.is_none());
        assert!(parsed.segment_id.is_none());
        assert_eq!(parsed.segment_ids.as_deref(), Some(&[][..]));
        assert_eq!(parsed.audience, Some(super::types::BroadcastAudience::All));
        assert_eq!(
            parsed.unsubscribe_scope,
            Some(super::types::BroadcastUnsubscribeScope::List)
        );
        assert_eq!(parsed.tracking, Some(false));
        assert_eq!(
            parsed.template_engine,
            Some(super::types::BroadcastTemplateEngine::Handlebars)
        );
        assert!(parsed.topic_id.is_none());
        assert_eq!(parsed.preview_text.as_deref(), Some("Preview"));
        assert_eq!(parsed.version_number, Some(1));
    }

    #[test]
    fn parse_cancel_broadcast_response_test() {
        let data = r#"{
    "object": "broadcast",
    "id": "498ee8e4-7aa2-4eb5-9f04-4194848049d1"
}"#;

        let _parsed =
            serde_json::from_str::<CancelBroadcastResponse>(data).expect("Parsing failed");
    }

    #[test]
    fn parse_recipients_response_sent_test() {
        let data = r#"{
    "object": "list",
    "has_more": false,
    "data": [
        {
            "id": "b2Zmc2V0OjA",
            "contact_id": "e169aa45-1ecf-4183-9955-b1499d5701d3",
            "email": "steve.wozniak@gmail.com"
        },
        {
            "id": "b2Zmc2V0OjE",
            "contact_id": null,
            "email": "dana@example.com"
        }
    ]
}"#;

        let parsed =
            serde_json::from_str::<crate::list_opts::ListResponse<BroadcastRecipient>>(data)
                .expect("Parsing failed");

        assert!(!parsed.has_more);
        assert_eq!(parsed.len(), 2);
        assert_eq!(
            parsed[0].contact_id.as_deref(),
            Some("e169aa45-1ecf-4183-9955-b1499d5701d3")
        );
        assert!(parsed[1].contact_id.is_none());
        assert!(parsed[0].count.is_none());
        assert!(parsed[0].bounce_type.is_none());
        assert!(parsed[0].clicked_links.is_none());
    }

    #[test]
    fn parse_recipients_response_opened_test() {
        let data = r#"{
    "id": "b2Zmc2V0OjA",
    "contact_id": "e169aa45-1ecf-4183-9955-b1499d5701d3",
    "email": "steve.wozniak@gmail.com",
    "count": 3
}"#;

        let parsed = serde_json::from_str::<BroadcastRecipient>(data).expect("Parsing failed");

        assert_eq!(parsed.count, Some(3));
        assert!(parsed.bounce_type.is_none());
        assert!(parsed.clicked_links.is_none());
    }

    #[test]
    fn parse_recipients_response_clicked_test() {
        let data = r#"{
    "id": "b2Zmc2V0OjA",
    "contact_id": "e169aa45-1ecf-4183-9955-b1499d5701d3",
    "email": "carter@example.com",
    "count": 3,
    "clicked_links": [
        { "url": "https://send.io/pricing", "clicks": 2 },
        { "url": "https://developer.send.io/docs/api", "clicks": 1 }
    ]
}"#;

        let parsed = serde_json::from_str::<BroadcastRecipient>(data).expect("Parsing failed");

        assert_eq!(parsed.count, Some(3));
        let clicked_links = parsed.clicked_links.expect("clicked_links should be set");
        assert_eq!(clicked_links.len(), 2);
        assert_eq!(clicked_links[0].url, "https://send.io/pricing");
        assert_eq!(clicked_links[0].clicks, 2);
    }

    #[test]
    fn parse_recipients_response_bounced_test() {
        let data = r#"{
    "id": "b2Zmc2V0OjA",
    "contact_id": null,
    "email": "bounced@example.com",
    "bounce_type": "permanent"
}"#;

        let parsed = serde_json::from_str::<BroadcastRecipient>(data).expect("Parsing failed");

        assert!(parsed.contact_id.is_none());
        assert!(parsed.count.is_none());
        assert!(parsed.clicked_links.is_none());
        assert_eq!(
            parsed.bounce_type,
            Some(super::types::BroadcastRecipientBounceType::Permanent)
        );
    }

    #[test]
    fn serialize_list_recipients_options_test() {
        use super::types::BroadcastRecipientBounceType;

        let opts = ListRecipientsOptions::new(BroadcastRecipientEventType::Bounced)
            .with_email("steve")
            .with_bounce_type(BroadcastRecipientBounceType::Permanent)
            .with_limit(10)
            .list_after("cursor-123");

        let json = serde_json::to_value(&opts).expect("Failed to serialize");

        assert_eq!(json["type"], "bounced");
        assert_eq!(json["email"], "steve");
        assert_eq!(json["bounce_type"], "permanent");
        assert_eq!(json["limit"], 10);
        assert_eq!(json["after"], "cursor-123");
        assert!(json.get("before").is_none() || json["before"].is_null());
    }

    #[tokio_shared_rt::test(shared = true)]
    #[serial_test::serial]
    #[cfg(not(feature = "blocking"))]
    #[ignore = "requires SENDIO_API_KEY and network access"]
    async fn recipients_not_found() -> DebugResult<()> {
        let sendio = &*CLIENT;
        std::thread::sleep(std::time::Duration::from_secs(1));

        let list_opts = ListRecipientsOptions::new(BroadcastRecipientEventType::Sent);
        let result = sendio
            .broadcasts
            .recipients("00000000-0000-0000-0000-000000000000", list_opts)
            .await;

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn parse_broadcast_clicked_links_test() {
        let data = r#"{
          "object": "list",
          "has_more": true,
          "data": [
            {
              "id": "b2Zmc2V0OjA",
              "url": "https://send.io/pricing",
              "clicks": 42,
              "unique_clicks": 30
            },
            {
              "id": "b2Zmc2V0OjE",
              "url": "https://developer.send.io/docs/api",
              "clicks": 17,
              "unique_clicks": 15
            }
          ]
        }"#;

        let _parsed =
            serde_json::from_str::<crate::list_opts::ListResponse<BroadcastClickedLink>>(data)
                .expect("Parsing failed");
    }
}

/// Requests made against a one-shot local HTTP server, so the exact method, path, query and
/// JSON body the client sends can be asserted.
#[cfg(test)]
#[cfg(not(feature = "blocking"))]
#[allow(clippy::indexing_slicing)]
mod mock_test {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::{Receiver, channel};

    use serde_json::{Value, json};

    use super::types::{
        BroadcastAudience, BroadcastTemplateEngine, BroadcastUnsubscribeScope,
        EstimateBroadcastOptions, ListBroadcastsOptions, UpdateBroadcastVersionOptions,
    };
    use crate::{
        ConfigBuilder, Sendio,
        list_opts::ListOptions,
        types::{
            BroadcastRecipientBounceType, BroadcastRecipientEventType, CreateBroadcastOptions,
            ListRecipientsOptions, SendBroadcastOptions, UpdateBroadcastOptions,
        },
    };

    const ID: &str = "498ee8e4-7aa2-4eb5-9f04-4194848049d1";
    const SEGMENT: &str = "fd644f07-a05a-467e-9bae-23bb7c35766a";

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

    fn id_response() -> Value {
        json!({ "object": "broadcast", "id": ID })
    }

    #[tokio::test]
    async fn create_draft_sends_empty_body() {
        let (sendio, rx) = mock(&id_response());

        let res = sendio
            .broadcasts
            .create(CreateBroadcastOptions::draft())
            .await
            .expect("create");
        assert_eq!(&*res.id, ID);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, "/broadcasts");
        assert_eq!(req.body, Some(json!({})));
    }

    #[tokio::test]
    async fn create_new_sends_segment_id_not_audience_id() {
        let (sendio, rx) = mock(&id_response());

        let opts =
            CreateBroadcastOptions::new(SEGMENT, "Acme <news@acme.test>", "Hello").with_text("Hi");
        let _res = sendio.broadcasts.create(opts).await.expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(
            req.body,
            Some(json!({
                "segment_id": SEGMENT,
                "from": "Acme <news@acme.test>",
                "subject": "Hello",
                "text": "Hi",
            }))
        );
    }

    #[tokio::test]
    async fn create_sends_every_field_and_tracking_false() {
        let (sendio, rx) = mock(&id_response());

        let opts = CreateBroadcastOptions::draft()
            .with_name("Launch")
            .with_from("Acme <news@acme.test>")
            .with_subject("Hello")
            .with_reply("reply@acme.test")
            .with_html("<p>Hi</p>")
            .with_text("Hi")
            .with_preview_text("Preview")
            .with_topic_id("topic_1")
            .with_segment_id(SEGMENT)
            .with_segment_ids(&["seg_2"])
            .with_exclude_segment_ids(&["seg_3"])
            .with_audience(BroadcastAudience::Segments)
            .with_unsubscribe_scope(BroadcastUnsubscribeScope::List)
            .with_tracking(false)
            .with_template_engine(BroadcastTemplateEngine::Handlebars)
            .with_send(true)
            .with_scheduled_at("in 1 hour");
        let _res = sendio.broadcasts.create(opts).await.expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(
            req.body,
            Some(json!({
                "name": "Launch",
                "from": "Acme <news@acme.test>",
                "subject": "Hello",
                "reply_to": ["reply@acme.test"],
                "html": "<p>Hi</p>",
                "text": "Hi",
                "preview_text": "Preview",
                "topic_id": "topic_1",
                "segment_id": SEGMENT,
                "segment_ids": ["seg_2"],
                "exclude_segment_ids": ["seg_3"],
                "audience": "segments",
                "unsubscribe_scope": "list",
                "tracking": false,
                "template_engine": "handlebars",
                "send": true,
                "scheduled_at": "in 1 hour",
            }))
        );
    }

    #[tokio::test]
    async fn create_all_contacts() {
        let (sendio, rx) = mock(&id_response());

        let opts = CreateBroadcastOptions::draft()
            .with_audience(BroadcastAudience::All)
            .with_unsubscribe_scope(BroadcastUnsubscribeScope::All)
            .with_template_engine(BroadcastTemplateEngine::Simple);
        let _res = sendio.broadcasts.create(opts).await.expect("create");

        let req = rx.recv().expect("request");
        assert_eq!(
            req.body,
            Some(
                json!({ "audience": "all", "unsubscribe_scope": "all", "template_engine": "simple" })
            )
        );
    }

    #[tokio::test]
    async fn update_sends_only_set_fields_and_no_id() {
        let (sendio, rx) = mock(&id_response());

        let opts = UpdateBroadcastOptions::new()
            .with_preview_text("New preview")
            .with_topic_id("topic_1")
            .with_segment_id(SEGMENT)
            .with_exclude_segment_ids(&["seg_3"])
            .with_tracking(false);
        let res = sendio.broadcasts.update(ID, opts).await.expect("update");
        assert_eq!(&*res.id, ID);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "PATCH");
        assert_eq!(req.target, format!("/broadcasts/{ID}"));
        assert_eq!(
            req.body,
            Some(json!({
                "preview_text": "New preview",
                "topic_id": "topic_1",
                "segment_id": SEGMENT,
                "exclude_segment_ids": ["seg_3"],
                "tracking": false,
            }))
        );
    }

    #[tokio::test]
    async fn update_name_only() {
        let (sendio, rx) = mock(&id_response());

        let opts = UpdateBroadcastOptions::new().with_name("Renamed");
        let _res = sendio.broadcasts.update(ID, opts).await.expect("update");

        let req = rx.recv().expect("request");
        assert_eq!(req.body, Some(json!({ "name": "Renamed" })));
    }

    #[tokio::test]
    async fn send_body_has_no_broadcast_id() {
        let (sendio, rx) = mock(&json!({ "id": ID }));

        let opts = SendBroadcastOptions::new(ID).with_scheduled_at("in 1 min");
        let res = sendio.broadcasts.send(opts).await.expect("send");
        assert_eq!(&*res.id, ID);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, format!("/broadcasts/{ID}/send"));
        assert_eq!(req.body, Some(json!({ "scheduled_at": "in 1 min" })));

        let (sendio, rx) = mock(&json!({ "id": ID }));
        let _res = sendio
            .broadcasts
            .send(SendBroadcastOptions::new(ID))
            .await
            .expect("send");
        assert_eq!(rx.recv().expect("request").body, Some(json!({})));
    }

    #[tokio::test]
    async fn list_with_pagination_only() {
        let (sendio, rx) = mock(&json!({ "object": "list", "has_more": false, "data": [] }));

        let res = sendio
            .broadcasts
            .list(ListOptions::default().with_limit(5).list_after("cursor_a"))
            .await
            .expect("list");
        assert!(res.is_empty());

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "GET");
        assert_eq!(req.target, "/broadcasts?limit=5&after=cursor_a");
    }

    #[tokio::test]
    async fn list_with_filters_and_nullable_fields() {
        let (sendio, rx) = mock(&json!({
            "object": "list",
            "has_more": true,
            "data": [{
                "id": ID,
                "name": null,
                "audience_id": null,
                "segment_id": null,
                "segment_ids": [],
                "exclude_segment_ids": [],
                "audience": "all",
                "status": "draft",
                "created_at": "2026-10-01T10:00:00+00:00",
                "scheduled_at": null,
                "sent_at": null,
                "failed_reason": null
            }]
        }));

        let opts = ListBroadcastsOptions::new()
            .with_status("draft")
            .with_segment_id(SEGMENT)
            .with_search("launch")
            .with_limit(10)
            .list_before("cursor_b");
        let res = sendio.broadcasts.list(opts).await.expect("list");
        assert!(res.has_more);
        assert!(res[0].name.is_none());
        assert!(res[0].audience_id.is_none());
        assert_eq!(res[0].audience, Some(BroadcastAudience::All));

        let req = rx.recv().expect("request");
        assert_eq!(
            req.target,
            format!(
                "/broadcasts?status=draft&segment_id={SEGMENT}&search=launch&limit=10&before=cursor_b"
            )
        );
    }

    #[tokio::test]
    async fn get_cancel_duplicate_delete_paths() {
        let (sendio, rx) = mock(&json!({
            "object": "broadcast",
            "id": ID,
            "name": "Launch",
            "audience_id": SEGMENT,
            "segment_id": SEGMENT,
            "status": "draft",
            "created_at": "2026-10-01T10:00:00+00:00",
            "scheduled_at": null,
            "sent_at": null,
            "from": null,
            "subject": null,
            "reply_to": null,
            "preview_text": null,
            "html": null,
            "text": null
        }));
        let broadcast = sendio.broadcasts.get(ID).await.expect("get");
        assert_eq!(broadcast.segment_id.as_deref(), Some(SEGMENT));
        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            ("GET", format!("/broadcasts/{ID}"))
        );

        let (sendio, rx) = mock(&id_response());
        let _res = sendio.broadcasts.cancel(ID).await.expect("cancel");
        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            ("POST", format!("/broadcasts/{ID}/cancel"))
        );

        let (sendio, rx) = mock(&json!({ "object": "broadcast", "id": "copy" }));
        let res = sendio.broadcasts.duplicate(ID).await.expect("duplicate");
        assert_eq!(&*res.id, "copy");
        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            ("POST", format!("/broadcasts/{ID}/duplicate"))
        );

        let (sendio, rx) = mock(&json!({ "object": "broadcast", "id": ID, "deleted": true }));
        assert!(sendio.broadcasts.delete(ID).await.expect("delete"));
        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            ("DELETE", format!("/broadcasts/{ID}"))
        );
    }

    #[tokio::test]
    async fn recipients_and_clicked_links_queries() {
        let (sendio, rx) = mock(&json!({ "object": "list", "has_more": false, "data": [] }));
        let opts = ListRecipientsOptions::new(BroadcastRecipientEventType::Bounced)
            .with_email("acme")
            .with_bounce_type(BroadcastRecipientBounceType::Transient)
            .with_limit(5)
            .list_after("cursor_a");
        let _res = sendio
            .broadcasts
            .recipients(ID, opts)
            .await
            .expect("recipients");
        let req = rx.recv().expect("request");
        assert_eq!(
            req.target,
            format!(
                "/broadcasts/{ID}/recipients?type=bounced&email=acme&bounce_type=transient&limit=5&after=cursor_a"
            )
        );

        let (sendio, rx) = mock(&json!({ "object": "list", "has_more": false, "data": [] }));
        let _res = sendio
            .broadcasts
            .clicked_links(ID, ListOptions::default().with_limit(3).list_before("c"))
            .await
            .expect("clicked links");
        let req = rx.recv().expect("request");
        assert_eq!(
            req.target,
            format!("/broadcasts/{ID}/clicked-links?limit=3&before=c")
        );
    }

    #[tokio::test]
    async fn estimate_sends_audience() {
        let (sendio, rx) = mock(&json!({
            "object": "broadcast_estimate",
            "recipients": 42,
            "skipped": { "invalid": 3 }
        }));

        let opts = EstimateBroadcastOptions::new()
            .with_segment_id(SEGMENT)
            .with_segment_ids(&["seg_2"])
            .with_exclude_segment_ids(&["seg_3"])
            .with_audience(BroadcastAudience::Segments)
            .with_topic_id("topic_1");
        let res = sendio.broadcasts.estimate(opts).await.expect("estimate");
        assert_eq!(res.recipients, 42);
        assert_eq!(res.skipped.invalid, 3);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, "/broadcasts/estimate");
        assert_eq!(
            req.body,
            Some(json!({
                "segment_id": SEGMENT,
                "segment_ids": ["seg_2"],
                "exclude_segment_ids": ["seg_3"],
                "audience": "segments",
                "topic_id": "topic_1",
            }))
        );
    }

    #[tokio::test]
    async fn test_sends_to() {
        let (sendio, rx) = mock(&json!({ "object": "broadcast_test", "sent": 2 }));

        let res = sendio
            .broadcasts
            .test(ID, &["a@acme.test", "b@acme.test"])
            .await
            .expect("test");
        assert_eq!(res.sent, 2);

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, format!("/broadcasts/{ID}/test"));
        assert_eq!(
            req.body,
            Some(json!({ "to": ["a@acme.test", "b@acme.test"] }))
        );
    }

    #[tokio::test]
    async fn stats() {
        let (sendio, rx) = mock(&json!({
            "object": "broadcast_stats",
            "status": "sent",
            "recipients": 10,
            "sent": 10,
            "delivered": 9,
            "bounced": 1,
            "complained": 0,
            "suppressed": 0,
            "failed": 0,
            "opened": 6,
            "unique_opened": 4,
            "clicked": 2,
            "unique_clicked": 2,
            "unsubscribed": 1,
            "skipped": { "invalid": 2 },
            "delivery_rate": 0.9,
            "open_rate": 0.4444,
            "click_rate": 0.2222,
            "bounce_rate": 0.1,
            "unsubscribe_rate": 0.1111,
            "progress": { "queued": 10, "total": 10 }
        }));

        let stats = sendio.broadcasts.stats(ID).await.expect("stats");
        assert_eq!(stats.status, "sent");
        assert_eq!(stats.delivered, 9);
        assert_eq!(stats.unique_opened, 4);
        assert_eq!(stats.skipped.invalid, 2);
        assert_eq!(stats.progress.total, 10);
        assert!((stats.open_rate - 0.4444).abs() < f64::EPSILON);

        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            ("GET", format!("/broadcasts/{ID}/stats"))
        );
    }

    #[tokio::test]
    async fn versions() {
        let summary = json!({
            "object": "broadcast_version",
            "id": "6c1d4e0e-0000-4000-8000-000000000002",
            "broadcast_id": ID,
            "version_number": 2,
            "name": null,
            "comment": null,
            "broadcast_name": "Launch",
            "subject": "Hello",
            "restored_from": 1,
            "created_at": "2026-10-01T10:00:00+00:00",
            "current": true
        });

        let (sendio, rx) = mock(&json!({ "object": "list", "has_more": true, "data": [summary] }));
        let res = sendio
            .broadcasts
            .list_versions(ID, ListOptions::default().with_limit(1))
            .await
            .expect("list versions");
        assert!(res.has_more);
        assert_eq!(res[0].version_number, 2);
        assert_eq!(res[0].restored_from, Some(1));
        assert!(res[0].current);
        let req = rx.recv().expect("request");
        assert_eq!(req.target, format!("/broadcasts/{ID}/versions?limit=1"));

        let mut detail = summary.clone();
        detail["tracking"] = json!(false);
        detail["template_engine"] = json!("handlebars");
        detail["segment_ids"] = json!([SEGMENT]);
        detail["audience"] = json!("segments");
        detail["topic_id"] = Value::Null;

        let (sendio, rx) = mock(&detail);
        let version = sendio
            .broadcasts
            .get_version(ID, 2)
            .await
            .expect("get version");
        assert_eq!(version.tracking, Some(false));
        assert_eq!(
            version.template_engine,
            Some(BroadcastTemplateEngine::Handlebars)
        );
        assert!(version.topic_id.is_none());
        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            ("GET", format!("/broadcasts/{ID}/versions/2"))
        );

        let (sendio, rx) = mock(&detail);
        let opts = UpdateBroadcastVersionOptions::new()
            .with_name("Final")
            .with_comment("Approved");
        let _version = sendio
            .broadcasts
            .update_version(ID, 2, opts)
            .await
            .expect("update version");
        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            ("PATCH", format!("/broadcasts/{ID}/versions/2"))
        );
        assert_eq!(
            req.body,
            Some(json!({ "name": "Final", "comment": "Approved" }))
        );

        let (sendio, rx) = mock(&detail);
        let _version = sendio
            .broadcasts
            .restore_version(ID, "6c1d4e0e-0000-4000-8000-000000000001")
            .await
            .expect("restore version");
        let req = rx.recv().expect("request");
        assert_eq!(
            (req.method.as_str(), req.target),
            (
                "POST",
                format!("/broadcasts/{ID}/versions/6c1d4e0e-0000-4000-8000-000000000001/restore")
            )
        );
    }
}
