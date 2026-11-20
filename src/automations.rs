use std::sync::Arc;

use reqwest::Method;

use crate::{
    Config, Result,
    idempotent::Idempotent,
    list_opts::{ListOptions, ListResponse},
    types::{
        Automation, AutomationMinimal, AutomationRun, AutomationTemplateCategory,
        AutomationTemplateDetail, CreateAutomationFromTemplateOptions,
        CreateAutomationFromTemplateResponse, CreateAutomationOptions, CreateAutomationResponse,
        DeleteAutomationResponse, DuplicateAutomationResponse, ListAutomationTemplatesResponse,
        StopAutomationResponse, UpdateAutomationOptions, UpdateAutomationResponse,
    },
};

/// `Sendio` APIs for `/automations` endpoints.
#[derive(Clone, Debug)]
pub struct AutomationsSvc {
    pub(crate) config: Arc<Config>,
    /// `Sendio` APIs for `/automation-templates` endpoints.
    pub templates: AutomationTemplatesSvc,
}

impl AutomationsSvc {
    pub(crate) fn new(config: Arc<Config>) -> Self {
        Self {
            templates: AutomationTemplatesSvc(Arc::clone(&config)),
            config,
        }
    }

    /// Set up an automation template in one transaction: the events your team does not have,
    /// the workflow's emails and the automation, which is created disabled.
    ///
    /// Add an `Idempotency-Key` with
    /// [`CreateAutomationFromTemplateOptions::with_idempotency_key`]: a retry with the same key
    /// returns the first answer instead of a second automation.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn create_from_template(
        &self,
        options: impl Into<Idempotent<CreateAutomationFromTemplateOptions>>,
    ) -> Result<CreateAutomationFromTemplateResponse> {
        let options: Idempotent<CreateAutomationFromTemplateOptions> = options.into();

        let mut request = self
            .config
            .build(Method::POST, "/automations/from-template");

        if let Some(ref idempotency_key) = options.idempotency_key {
            request = request.header("Idempotency-Key", idempotency_key);
        }

        let response = self.config.send(request.json(&options)).await?;
        let content = response
            .json::<CreateAutomationFromTemplateResponse>()
            .await?;

        Ok(content)
    }

    /// Create a new automation to automate email sequences.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        automation: CreateAutomationOptions,
    ) -> Result<CreateAutomationResponse> {
        let request = self.config.build(Method::POST, "/automations");
        let response = self.config.send(request.json(&automation)).await?;
        let content = response.json::<CreateAutomationResponse>().await?;

        Ok(content)
    }

    /// Update an existing automation.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        automation_id: &str,
        update: UpdateAutomationOptions,
    ) -> Result<UpdateAutomationResponse> {
        let path = format!("/automations/{automation_id}");

        let request = self.config.build(Method::PATCH, &path);
        let response = self.config.send(request.json(&update)).await?;
        let content = response.json::<UpdateAutomationResponse>().await?;

        Ok(content)
    }

    /// Retrieve a single automation.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get(&self, automation_id: &str) -> Result<Automation> {
        let path = format!("/automations/{automation_id}");

        let request = self.config.build(Method::GET, &path);
        let response = self.config.send(request).await?;
        let content = response.json::<Automation>().await?;

        Ok(content)
    }

    /// Retrieve a list of automations.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list<T>(
        &self,
        list_opts: ListOptions<T>,
    ) -> Result<ListResponse<AutomationMinimal>> {
        let request = self
            .config
            .build(Method::GET, "/automations")
            .query(&list_opts);
        let response = self.config.send(request).await?;
        let content = response.json::<ListResponse<AutomationMinimal>>().await?;

        Ok(content)
    }

    /// Stop a running automation.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn stop(&self, automation_id: &str) -> Result<StopAutomationResponse> {
        let path = format!("/automations/{automation_id}/stop");

        let request = self.config.build(Method::POST, &path);
        let response = self.config.send(request).await?;
        let content = response.json::<StopAutomationResponse>().await?;

        Ok(content)
    }

    /// Duplicate an existing automation.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn duplicate(&self, automation_id: &str) -> Result<DuplicateAutomationResponse> {
        let path = format!("/automations/{automation_id}/duplicate");

        let request = self.config.build(Method::POST, &path);
        let response = self.config.send(request).await?;
        let content = response.json::<DuplicateAutomationResponse>().await?;

        Ok(content)
    }

    /// Remove an existing automation.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn delete(&self, automation_id: &str) -> Result<DeleteAutomationResponse> {
        let path = format!("/automations/{automation_id}");

        let request = self.config.build(Method::DELETE, &path);
        let response = self.config.send(request).await?;
        let content = response.json::<DeleteAutomationResponse>().await?;

        Ok(content)
    }

    /// Retrieve a list of automation runs.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list_runs<T>(
        &self,
        automation_id: &str,
        status_filter: Option<String>,
        list_opts: ListOptions<T>,
    ) -> Result<ListResponse<AutomationRun>> {
        let path = format!("/automations/{automation_id}/runs");

        let request = self
            .config
            .build(Method::GET, &path)
            .query(&list_opts)
            .query(&status_filter);
        let response = self.config.send(request).await?;
        let content = response.json::<ListResponse<AutomationRun>>().await?;

        Ok(content)
    }

    /// Retrieve a single automation run.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get_run(&self, automation_id: &str, run_id: &str) -> Result<AutomationRun> {
        let path = format!("/automations/{automation_id}/runs/{run_id}");

        let request = self.config.build(Method::GET, &path);
        let response = self.config.send(request).await?;
        let content = response.json::<AutomationRun>().await?;

        Ok(content)
    }
}

/// `Sendio` APIs for `/automation-templates` endpoints.
///
/// Automation templates are prebuilt workflows with their emails, owned by send.io. Set one up
/// with [`AutomationsSvc::create_from_template`].
#[derive(Clone, Debug)]
pub struct AutomationTemplatesSvc(pub(crate) Arc<Config>);

impl AutomationTemplatesSvc {
    /// Retrieve the automation templates, optionally of one category. This endpoint is not
    /// paginated.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        category: Option<AutomationTemplateCategory>,
    ) -> Result<ListAutomationTemplatesResponse> {
        let mut request = self.0.build(Method::GET, "/automation-templates");

        if let Some(category) = category {
            request = request.query(&[("category", category)]);
        }

        let response = self.0.send(request).await?;
        let content = response.json::<ListAutomationTemplatesResponse>().await?;

        Ok(content)
    }

    /// Retrieve one automation template in full by slug: its steps, emails (filled with your
    /// team's brand), the params setup asks for and each event's schema.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get(&self, slug: &str) -> Result<AutomationTemplateDetail> {
        let path = format!("/automation-templates/{slug}");

        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<AutomationTemplateDetail>().await?;

        Ok(content)
    }
}

#[allow(unreachable_pub)]
pub mod types {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::idempotent::Idempotent;

    crate::define_id_type!(AutomationId);
    crate::define_id_type!(AutomationRunId);

    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    pub struct CreateAutomationOptions {
        pub name: String,
        pub status: AutomationStatus,
        pub steps: Vec<Step>,
        pub connections: Vec<Connection>,
    }

    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
    #[serde(rename_all = "snake_case")]
    pub enum AutomationStatus {
        Enabled,
        #[default]
        Disabled,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Step {
        Trigger {
            key: String,
            config: TriggerStepConfig,
        },
        SendEmail {
            key: String,
            config: SendEmailStepConfig,
        },
        Delay {
            key: String,
            config: DelayStepConfig,
        },
        WaitForEvent {
            key: String,
            config: WaitForEventStepConfig,
        },
        Condition {
            key: String,
            config: Value,
        },
        ContactUpdate {
            key: String,
            config: Value,
        },
        ContactDelete {
            key: String,
            config: Value,
        },
        AddToSegment {
            key: String,
            config: AddToSegmentStepConfig,
        },
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TriggerStepConfig {
        pub event_name: String,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SendEmailStepConfig {
        pub template: AutomationTemplate,
        pub subject: Option<String>,
        pub from: Option<String>,
        pub reply_to: Option<String>,
        pub variables: Option<Value>,
    }

    impl SendEmailStepConfig {
        #[inline]
        pub fn new(template: AutomationTemplate) -> Self {
            Self {
                template,
                subject: None,
                from: None,
                reply_to: None,
                variables: None,
            }
        }

        #[inline]
        pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
            self.subject = Some(subject.into());
            self
        }

        #[inline]
        pub fn with_from(mut self, from: impl Into<String>) -> Self {
            self.from = Some(from.into());
            self
        }

        #[inline]
        pub fn with_reply_to(mut self, reply_to: impl Into<String>) -> Self {
            self.reply_to = Some(reply_to.into());
            self
        }

        #[inline]
        pub fn with_variables(mut self, variables: Value) -> Self {
            self.variables = Some(variables);
            self
        }
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct AutomationTemplate {
        pub id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub variables: Option<Value>,
    }

    impl AutomationTemplate {
        #[inline]
        pub fn new(id: impl Into<String>) -> Self {
            Self {
                id: id.into(),
                variables: None,
            }
        }

        #[inline]
        pub fn with_variables(mut self, variables: Value) -> Self {
            self.variables = Some(variables);
            self
        }
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DelayStepConfig {
        pub duration: String,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WaitForEventStepConfig {
        pub event_name: String,
        pub timeout: Option<String>,
        pub filter_rule: Option<Value>,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddToSegmentStepConfig {
        pub segment_id: String,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Connection {
        pub from: String,
        pub to: String,
        pub r#type: Option<ConnectionType>,
    }

    impl Connection {
        #[inline]
        pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
            Self {
                from: from.into(),
                to: to.into(),
                r#type: None,
            }
        }

        #[inline]
        pub fn with_type(mut self, r#type: ConnectionType) -> Self {
            self.r#type = Some(r#type);
            self
        }
    }

    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
    #[serde(rename_all = "snake_case")]
    pub enum ConnectionType {
        #[default]
        Default,
        ConditionMet,
        ConditionNotMet,
        Timeout,
        EventReceived,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateAutomationResponse {
        pub id: AutomationId,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Automation {
        pub id: AutomationId,
        pub name: String,
        pub status: AutomationStatus,
        pub created_at: String,
        pub updated_at: Option<String>,
        pub steps: Vec<Step>,
        pub connections: Vec<Connection>,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationMinimal {
        pub id: AutomationId,
        pub name: String,
        pub status: AutomationStatus,
        pub created_at: String,
        pub updated_at: Option<String>,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Default)]
    pub struct UpdateAutomationOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<AutomationStatus>,
        #[serde(skip_serializing_if = "Option::is_none")]
        steps: Option<Vec<Step>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        connections: Option<Vec<Connection>>,
    }

    impl UpdateAutomationOptions {
        #[inline]
        pub fn new() -> Self {
            Self::default()
        }

        #[inline]
        pub fn with_name(mut self, name: &str) -> Self {
            self.name = Some(name.to_owned());
            self
        }

        #[inline]
        pub fn with_status(mut self, status: AutomationStatus) -> Self {
            self.status = Some(status);
            self
        }

        #[inline]
        pub fn with_steps(mut self, steps: Vec<Step>) -> Self {
            self.steps = Some(steps);
            self
        }

        #[inline]
        pub fn with_connections(mut self, connections: Vec<Connection>) -> Self {
            self.connections = Some(connections);
            self
        }
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateAutomationResponse {
        pub id: AutomationId,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StopAutomationResponse {
        pub id: AutomationId,
        pub status: AutomationStatus,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DuplicateAutomationResponse {
        pub id: AutomationId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeleteAutomationResponse {
        pub id: AutomationId,
        pub deleted: bool,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationRun {
        id: AutomationRunId,
        #[serde(skip_serializing_if = "Option::is_none")]
        started_at: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        completed_at: Option<String>,
        created_at: String,
        status: AutomationRunStatus,
        trigger: Option<AutomationRunTrigger>,
    }

    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum AutomationRunStatus {
        Running,
        Completed,
        Failed,
        Cancelled,
    }

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationRunTrigger {
        event_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<HashMap<String, Value>>,
    }

    /// The category of an automation template.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum AutomationTemplateCategory {
        Onboarding,
        Billing,
        Ecommerce,
        Engagement,
        LeadCapture,
        Internal,
        /// A value this version of the SDK does not know.
        #[serde(other)]
        Unknown,
    }

    /// Whether an automation template's emails are marketing (they need a topic) or
    /// transactional.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum AutomationTemplateKind {
        Marketing,
        Transactional,
        /// A value this version of the SDK does not know.
        #[serde(other)]
        Unknown,
    }

    /// The type of a value setup asks for.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum AutomationTemplateParamType {
        /// A number and minutes, hours, days or weeks, e.g. `2 days`. Has a default.
        Duration,
        /// A full http or https link. Has no default and must be given.
        Url,
        /// A value this version of the SDK does not know.
        #[serde(other)]
        Unknown,
    }

    /// An event an automation template uses.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateEvent {
        pub r#ref: String,
        pub name: String,
        pub purpose: String,
        /// Whether your team already has the event.
        pub exists: bool,
        /// The event's fields and their types. Only returned by `templates.get`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub schema: Option<HashMap<String, String>>,
        /// The fields setup would add to your team's event. Only returned by `templates.get`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub adds: Option<HashMap<String, String>>,
    }

    /// A step of an automation template. `config` is only returned by `templates.get`.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateStep {
        pub key: String,
        pub r#type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub config: Option<Value>,
    }

    /// One of an automation template's emails, filled with your team's brand as setup will
    /// create it.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateEmail {
        pub r#ref: String,
        pub step_label: String,
        pub step_key: String,
        pub subject: String,
        pub preview: String,
        pub html: String,
        pub text: String,
        #[serde(default)]
        pub variables: Vec<Value>,
    }

    /// A value setup asks for, set with [`CreateAutomationFromTemplateOptions::with_param`].
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateParam {
        pub r#ref: String,
        pub r#type: AutomationTemplateParamType,
        pub label: String,
        pub default: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub placeholder: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub hint: Option<String>,
    }

    /// An automation template as listed by `automations.templates.list`.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateSummary {
        pub slug: String,
        pub version: u32,
        pub name: String,
        pub category: AutomationTemplateCategory,
        pub kind: AutomationTemplateKind,
        pub outcome: String,
        pub events: Vec<AutomationTemplateEvent>,
        pub email_count: u32,
        pub steps: Vec<AutomationTemplateStep>,
        pub connections: Vec<Connection>,
        /// How many of your automations were made from the template.
        pub in_use_count: u32,
    }

    /// The response of `automations.templates.list`.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ListAutomationTemplatesResponse {
        pub categories: Vec<AutomationTemplateCategory>,
        pub data: Vec<AutomationTemplateSummary>,
    }

    /// One automation template in full, as returned by `automations.templates.get`.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateDetail {
        pub slug: String,
        pub version: u32,
        pub name: String,
        pub category: AutomationTemplateCategory,
        pub kind: AutomationTemplateKind,
        pub outcome: String,
        pub how_it_works: String,
        pub events: Vec<AutomationTemplateEvent>,
        pub email_count: u32,
        pub emails: Vec<AutomationTemplateEmail>,
        pub params: Vec<AutomationTemplateParam>,
        pub steps: Vec<AutomationTemplateStep>,
        pub connections: Vec<Connection>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub settings: Option<Value>,
        pub in_use_count: u32,
    }

    /// How an automation template's event ref is mapped.
    #[must_use]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
    #[serde(rename_all = "snake_case")]
    pub enum AutomationTemplateEventMode {
        /// Use the event, creating it if your team does not have it.
        #[default]
        Create,
        /// Map the ref to an event your app already sends.
        Existing,
    }

    /// The event to use for one of an automation template's event refs.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateEventChoice {
        pub mode: AutomationTemplateEventMode,
        pub name: String,
    }

    impl AutomationTemplateEventChoice {
        /// Use (or create) the event `name`.
        #[inline]
        pub fn create(name: impl Into<String>) -> Self {
            Self {
                mode: AutomationTemplateEventMode::Create,
                name: name.into(),
            }
        }

        /// Use the event `name` your app already sends.
        #[inline]
        pub fn existing(name: impl Into<String>) -> Self {
            Self {
                mode: AutomationTemplateEventMode::Existing,
                name: name.into(),
            }
        }
    }

    /// The sender of an automation template's emails.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationTemplateSender {
        /// Must be on one of your verified sending domains.
        pub from: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reply_to: Option<String>,
    }

    /// The body of `automations.create_from_template`.
    #[must_use]
    #[derive(Debug, Clone, Serialize)]
    pub struct CreateAutomationFromTemplateOptions {
        slug: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        events: Option<HashMap<String, AutomationTemplateEventChoice>>,
        sender: AutomationTemplateSender,
        #[serde(skip_serializing_if = "Option::is_none")]
        topic_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<HashMap<String, String>>,
    }

    impl CreateAutomationFromTemplateOptions {
        /// Set up the template `slug`, sending its emails from `from`.
        #[inline]
        pub fn new(slug: impl Into<String>, from: impl Into<String>) -> Self {
            Self {
                slug: slug.into(),
                version: None,
                name: None,
                events: None,
                sender: AutomationTemplateSender {
                    from: from.into(),
                    reply_to: None,
                },
                topic_id: None,
                params: None,
            }
        }

        /// The version you reviewed. A newer one returns 409 `template_version_changed`.
        #[inline]
        pub fn with_version(mut self, version: u32) -> Self {
            self.version = Some(version);
            self
        }

        /// The automation's name, 1 to 120 characters. Default: the template's name.
        #[inline]
        pub fn with_name(mut self, name: impl Into<String>) -> Self {
            self.name = Some(name.into());
            self
        }

        /// Map the template's event `event_ref` to an event.
        #[inline]
        pub fn with_event(
            mut self,
            event_ref: impl Into<String>,
            choice: AutomationTemplateEventChoice,
        ) -> Self {
            let _prev = self
                .events
                .get_or_insert_with(HashMap::new)
                .insert(event_ref.into(), choice);
            self
        }

        #[inline]
        pub fn with_reply_to(mut self, reply_to: impl Into<String>) -> Self {
            self.sender.reply_to = Some(reply_to.into());
            self
        }

        /// Required for a marketing template: the topic contacts can unsubscribe from.
        #[inline]
        pub fn with_topic_id(mut self, topic_id: impl Into<String>) -> Self {
            self.topic_id = Some(topic_id.into());
            self
        }

        /// Set the param `param_ref`: a duration such as `2 days`, or a full http(s) link.
        #[inline]
        pub fn with_param(
            mut self,
            param_ref: impl Into<String>,
            value: impl Into<String>,
        ) -> Self {
            let _prev = self
                .params
                .get_or_insert_with(HashMap::new)
                .insert(param_ref.into(), value.into());
            self
        }

        /// Adds an `Idempotency-Key` header to the request: a retry with the same key returns
        /// the first answer instead of a second automation.
        #[inline]
        pub fn with_idempotency_key(self, idempotency_key: &str) -> Idempotent<Self> {
            Idempotent {
                idempotency_key: Some(idempotency_key.to_owned()),
                data: self,
            }
        }
    }

    /// The template an automation was made from.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationSourceTemplate {
        pub slug: String,
        pub version: u32,
        pub name: String,
    }

    /// A workflow email owned by an automation.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AutomationEmail {
        pub id: String,
        pub name: String,
        pub status: String,
        pub step_key: String,
    }

    /// The response of `automations.create_from_template`. The automation is created disabled.
    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateAutomationFromTemplateResponse {
        pub id: AutomationId,
        pub status: AutomationStatus,
        pub source_template: AutomationSourceTemplate,
        pub emails: Vec<AutomationEmail>,
        /// The events setup created.
        pub events_created: Vec<String>,
        /// The events your team already had.
        pub events_reused: Vec<String>,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::needless_return)]
mod test {
    use crate::{
        automations::types::{SendEmailStepConfig, TriggerStepConfig},
        types::{Automation, AutomationStatus, Connection, CreateAutomationOptions, Step},
    };

    #[cfg(not(feature = "blocking"))]
    use crate::{
        list_opts::ListOptions,
        test::{CLIENT, DebugResult},
        types::UpdateAutomationOptions,
    };

    #[test]
    fn serialize_create() {
        let tmp = CreateAutomationOptions {
            name: "Welcome series".to_owned(),
            status: AutomationStatus::Enabled,
            steps: vec![
                Step::Trigger {
                    key: "start".to_owned(),
                    config: TriggerStepConfig {
                        event_name: "user.created".to_owned(),
                    },
                },
                Step::SendEmail {
                    key: "welcome".to_owned(),
                    config: SendEmailStepConfig {
                        subject: None,
                        from: None,
                        reply_to: None,
                        variables: None,
                        template: crate::automations::types::AutomationTemplate {
                            id: "34a080c9-b17d-4187-ad80-5af20266e535".to_owned(),
                            variables: None,
                        },
                    },
                },
            ],
            connections: vec![Connection {
                from: "start".to_owned(),
                to: "welcome".to_owned(),
                r#type: None,
            }],
        };

        println!("{}", serde_json::to_string(&tmp).unwrap());
    }

    #[test]
    fn deserialize_get() {
        let tmp = r#"
          {
            "object": "automation",
            "id": "c9b16d4f-ba6c-4e2e-b044-6bf4404e57fd",
            "name": "Welcome series",
            "status": "disabled",
            "created_at": "2026-10-01T12:00:00+00:00",
            "updated_at": "2026-10-01T12:00:00+00:00",
            "steps": [
              {
                "key": "start",
                "type": "trigger",
                "config": { "event_name": "user.created" }
              },
              {
                "key": "welcome",
                "type": "send_email",
                "config": {
                  "template": { "id": "34a080c9-b17d-4187-ad80-5af20266e535" }
                }
              }
            ],
            "connections": [
              {
                "from": "start",
                "to": "welcome",
                "type": "default"
              }
            ]
          }"#;

        let _res = serde_json::from_str::<Automation>(tmp).unwrap();
    }

    #[tokio_shared_rt::test(shared = true)]
    #[serial_test::serial]
    #[cfg(not(feature = "blocking"))]
    #[ignore = "live API test; run with `cargo test -- --ignored` and SENDIO_API_KEY set"]
    async fn all() -> DebugResult<()> {
        let sendio = &*CLIENT;

        // Create
        let opts = CreateAutomationOptions {
            name: "Welcome series".to_owned(),
            status: AutomationStatus::Enabled,
            steps: vec![Step::Trigger {
                key: "trigger".to_owned(),
                config: TriggerStepConfig {
                    event_name: "user.created".to_owned(),
                },
            }],
            connections: vec![],
        };
        let automation = sendio.automations.create(opts).await?;
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Update
        let opts = UpdateAutomationOptions::new().with_status(AutomationStatus::Enabled);
        let automation = sendio.automations.update(&automation.id, opts).await?;

        // Get
        let automation = sendio.automations.get(&automation.id).await?;

        // List
        let automations = sendio.automations.list(ListOptions::default()).await?;
        assert!(!automations.data.is_empty());

        // List Runs
        let runs = sendio
            .automations
            .list_runs(&automation.id, None, ListOptions::default())
            .await?;
        assert!(runs.data.is_empty());

        // Duplicate
        let duplicated = sendio.automations.duplicate(&automation.id).await?;
        std::thread::sleep(std::time::Duration::from_secs(2));
        let deleted = sendio.automations.delete(&duplicated.id).await?;
        assert!(deleted.deleted);

        // Stop
        let automation = sendio.automations.stop(&automation.id).await?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Delete
        let automation = sendio.automations.delete(&automation.id).await?;
        assert!(automation.deleted);

        Ok(())
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

    use super::types::{
        AutomationStatus, AutomationTemplateCategory, AutomationTemplateEventChoice,
        AutomationTemplateKind, AutomationTemplateParamType, CreateAutomationFromTemplateOptions,
    };
    use crate::{ConfigBuilder, Sendio};

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

    fn summary() -> Value {
        json!({
            "object": "automation_template",
            "slug": "email-verification-reminder",
            "version": 1,
            "name": "Email verification reminder",
            "category": "onboarding",
            "kind": "transactional",
            "outcome": "Remind users who haven't verified their email, at most twice.",
            "events": [
                { "ref": "trigger", "name": "user.created", "purpose": "Starts the workflow.", "exists": true },
                { "ref": "verified", "name": "user.email.verified", "purpose": "Ends it.", "exists": false },
            ],
            "email_count": 2,
            "steps": [
                { "key": "trigger", "type": "trigger" },
                { "key": "wait_1", "type": "wait_for_event" },
            ],
            "connections": [{ "from": "trigger", "to": "wait_1", "type": "default" }],
            "in_use_count": 1,
        })
    }

    fn created() -> Value {
        json!({
            "object": "automation",
            "id": "c9b16d4f-ba6c-4e2e-b044-6bf4404e57fd",
            "status": "disabled",
            "source_template": { "slug": "email-verification-reminder", "version": 2, "name": "Email verification reminder" },
            "emails": [{ "object": "automation_email", "id": "tpl_1", "name": "Reminder 1", "status": "published", "step_key": "email_1" }],
            "events_created": ["user.email.verified"],
            "events_reused": ["user.created"],
        })
    }

    #[tokio::test]
    async fn list_templates() {
        let (sendio, rx) = mock(
            "200 OK",
            &json!({ "object": "list", "categories": ["onboarding", "lead_capture"], "data": [summary()] }),
        );

        let list = sendio.automations.templates.list(None).await.expect("list");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "GET");
        assert_eq!(req.target, "/automation-templates");
        assert_eq!(req.body, None);

        assert_eq!(
            list.categories,
            vec![
                AutomationTemplateCategory::Onboarding,
                AutomationTemplateCategory::LeadCapture
            ]
        );
        let tpl = &list.data[0];
        assert_eq!(tpl.slug, "email-verification-reminder");
        assert_eq!(tpl.version, 1);
        assert_eq!(tpl.category, AutomationTemplateCategory::Onboarding);
        assert_eq!(tpl.kind, AutomationTemplateKind::Transactional);
        assert_eq!(tpl.email_count, 2);
        assert_eq!(tpl.in_use_count, 1);
        assert!(tpl.events[0].exists);
        assert!(!tpl.events[1].exists);
        assert_eq!(tpl.events[1].schema, None);
        assert_eq!(tpl.steps[1].r#type, "wait_for_event");
        assert_eq!(tpl.connections[0].to, "wait_1");
    }

    #[tokio::test]
    async fn list_templates_by_category() {
        let (sendio, rx) = mock(
            "200 OK",
            &json!({ "object": "list", "categories": ["lead_capture"], "data": [] }),
        );

        let list = sendio
            .automations
            .templates
            .list(Some(AutomationTemplateCategory::LeadCapture))
            .await
            .expect("list");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "GET");
        assert_eq!(req.target, "/automation-templates?category=lead_capture");
        assert!(list.data.is_empty());
    }

    #[tokio::test]
    async fn get_template() {
        let mut detail = summary();
        detail["version"] = json!(2);
        detail["how_it_works"] = json!("Each signup starts its own run for that user.");
        detail["events"] = json!([{
            "ref": "verified", "name": "user.email.verified", "purpose": "Ends it.",
            "schema": { "user_id": "string" }, "exists": false, "adds": { "user_id": "string" },
        }]);
        detail["emails"] = json!([{
            "ref": "reminder_1", "step_label": "Reminder 1", "step_key": "email_1",
            "subject": "Confirm your email", "preview": "One click", "html": "<p>Hi</p>",
            "text": "Hi", "variables": [{ "key": "verify_url" }],
        }]);
        detail["params"] = json!([
            { "ref": "wait_1", "type": "duration", "label": "First reminder after", "default": "24 hours" },
            { "ref": "resend_url", "type": "url", "label": "Link", "default": "", "placeholder": "https://app.example.com/verify-email" },
        ]);
        detail["steps"] = json!([{ "key": "trigger", "type": "trigger", "config": { "event_name": "user.created" } }]);
        detail["settings"] = json!({ "exit_on": "verified" });
        let (sendio, rx) = mock("200 OK", &detail);

        let tpl = sendio
            .automations
            .templates
            .get("email-verification-reminder")
            .await
            .expect("get");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "GET");
        assert_eq!(
            req.target,
            "/automation-templates/email-verification-reminder"
        );
        assert_eq!(req.body, None);

        assert_eq!(tpl.version, 2);
        assert_eq!(
            tpl.how_it_works,
            "Each signup starts its own run for that user."
        );
        let schema = tpl.events[0].schema.as_ref().expect("schema");
        assert_eq!(schema["user_id"], "string");
        assert_eq!(
            tpl.events[0].adds.as_ref().expect("adds")["user_id"],
            "string"
        );
        assert_eq!(tpl.emails[0].step_key, "email_1");
        assert_eq!(tpl.emails[0].html, "<p>Hi</p>");
        assert_eq!(
            tpl.emails[0].variables,
            vec![json!({ "key": "verify_url" })]
        );
        assert_eq!(tpl.params[0].r#type, AutomationTemplateParamType::Duration);
        assert_eq!(tpl.params[0].default, "24 hours");
        assert_eq!(tpl.params[1].r#type, AutomationTemplateParamType::Url);
        assert_eq!(
            tpl.params[1].placeholder.as_deref(),
            Some("https://app.example.com/verify-email")
        );
        assert_eq!(tpl.params[1].hint, None);
        assert_eq!(
            tpl.steps[0].config,
            Some(json!({ "event_name": "user.created" }))
        );
        assert_eq!(tpl.settings, Some(json!({ "exit_on": "verified" })));
    }

    #[tokio::test]
    async fn create_from_template_sends_every_field() {
        let (sendio, rx) = mock("201 Created", &created());

        let opts = CreateAutomationFromTemplateOptions::new(
            "email-verification-reminder",
            "Acme <hello@acme.com>",
        )
        .with_version(2)
        .with_name("Verify reminders")
        .with_event(
            "verified",
            AutomationTemplateEventChoice::existing("account.confirmed"),
        )
        .with_reply_to("help@acme.com")
        .with_topic_id("top_1")
        .with_param("wait_1", "24 hours")
        .with_param("resend_url", "https://app.acme.com/verify-email");
        let res = sendio
            .automations
            .create_from_template(opts)
            .await
            .expect("create from template");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, "/automations/from-template");
        assert_eq!(req.idempotency_key, None);
        assert_eq!(
            req.body,
            Some(json!({
                "slug": "email-verification-reminder",
                "version": 2,
                "name": "Verify reminders",
                "events": { "verified": { "mode": "existing", "name": "account.confirmed" } },
                "sender": { "from": "Acme <hello@acme.com>", "reply_to": "help@acme.com" },
                "topic_id": "top_1",
                "params": { "wait_1": "24 hours", "resend_url": "https://app.acme.com/verify-email" },
            }))
        );

        assert_eq!(&*res.id, "c9b16d4f-ba6c-4e2e-b044-6bf4404e57fd");
        assert!(matches!(res.status, AutomationStatus::Disabled));
        assert_eq!(res.source_template.slug, "email-verification-reminder");
        assert_eq!(res.source_template.version, 2);
        assert_eq!(res.emails[0].step_key, "email_1");
        assert_eq!(res.emails[0].status, "published");
        assert_eq!(res.events_created, vec!["user.email.verified"]);
        assert_eq!(res.events_reused, vec!["user.created"]);
    }

    #[tokio::test]
    async fn create_from_template_with_idempotency_key() {
        let (sendio, rx) = mock("201 Created", &created());

        let opts = CreateAutomationFromTemplateOptions::new(
            "email-verification-reminder",
            "hello@acme.com",
        )
        .with_event("trigger", AutomationTemplateEventChoice::create("signup"))
        .with_idempotency_key("setup-verify-1");
        let _res = sendio
            .automations
            .create_from_template(opts)
            .await
            .expect("create from template");

        let req = rx.recv().expect("request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, "/automations/from-template");
        assert_eq!(req.idempotency_key.as_deref(), Some("setup-verify-1"));
        // Unset optional fields are left out; the key is a header, not a body field.
        assert_eq!(
            req.body,
            Some(json!({
                "slug": "email-verification-reminder",
                "events": { "trigger": { "mode": "create", "name": "signup" } },
                "sender": { "from": "hello@acme.com" },
            }))
        );
    }

    #[test]
    fn unknown_template_enum_values_deserialize() {
        let mut value = summary();
        value["category"] = json!("retention");
        value["kind"] = json!("hybrid");

        let tpl = serde_json::from_value::<super::types::AutomationTemplateSummary>(value)
            .expect("parse");
        assert_eq!(tpl.category, AutomationTemplateCategory::Unknown);
        assert_eq!(tpl.kind, AutomationTemplateKind::Unknown);
    }
}
