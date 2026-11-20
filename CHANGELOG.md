# Changelog

## 1.2.0 (2026-11-18)

- New `sendio.transactional` service (`TransactionalSvc`, `/transactional`): `create`
  (`CreateTransactionalEmailOptions::new(name, template, from, subject)`, `with_idempotency_key`), `list`
  (`ListTransactionalEmailsOptions` with `with_search`, `with_template_id` and pagination), `get` (with the published
  version's `variables`), `update` (`with_expected_updated_at` for a 409 on a stale save), `delete`, `send_test` and
  `stats`. `with_track_opens` / `with_track_clicks` are off by default.
- `CreateEmailBaseOptions::transactional(transactional_id, to)` sends a transactional email, with `with_variable`;
  `with_track_opens` / `with_track_clicks` override tracking for one email. An empty `from` or `subject` is now left
  out of the request (a template or transactional email supplies it). `Email::transactional_id` reads it back.
- Broadcasts: `with_template_id` and `with_template_version_id` on create and update,
  `UpdateBroadcastOptions::detach_template` (`template_id: null`), and the template link fields and
  `unfillable_variables` on `Broadcast`.
- Deprecated: sending a template with `CreateEmailBaseOptions::with_template` (a template id). Send a transactional email with `CreateEmailBaseOptions::transactional` instead; the old way still works for now.
- `Mailbox` reads `can_send` and `send_reason` (whether the address is accepted as `from`, and why not).
- `Webhook` reads `signing_secret_hidden`, `disabled_reason` and `failing_since`; `signing_secret` is now optional (left out for workspace Members).
- User-Agent `sendio-rust:1.2.0`.

## 1.1.0 (2026-11-11)

First public release.

- Automation templates: new `sendio.automations.templates` (`AutomationTemplatesSvc`) with `list(category)`
  (`Option<AutomationTemplateCategory>`, not paginated) and `get(slug)` (`AutomationTemplateDetail`: emails, params,
  event schemas). `sendio.automations.create_from_template(CreateAutomationFromTemplateOptions::new(slug, from))`
  sets one up (`POST /automations/from-template`) with `with_version`, `with_name`, `with_event`
  (`AutomationTemplateEventChoice::create`/`::existing`), `with_reply_to`, `with_topic_id`, `with_param`, and
  `with_idempotency_key` for an `Idempotency-Key` header. `AutomationsSvc` is now a struct with a `templates` field
  instead of a tuple struct (its inner field was crate-private, so no public API changes).
- Removed `domains.claim`, `domains.get_claim` and `domains.verify_claim` and the `DomainClaim` types: the API has no domain-claim routes, so every call failed with a 404. Claim a domain by deleting it from the team that owns it, then adding it to yours.
- Domain receiving: new `DomainCapabilitiesOptions` (`with_receiving`, `with_sending`, taking `DomainCapabilityStatus`).
  `DomainChanges::with_capabilities` turns receiving on or off on `domains.update` (`Disabled` is sent).
  `CreateDomainOptions::with_capabilities` now takes `impl Into<Value>`, so it accepts `DomainCapabilitiesOptions` as
  well as the raw `Value` it took before. Off stops send.io accepting email for the domain (remove its MX record); on
  in an update asks for the domain's MX record (publish it to start receiving email); sending can't be disabled.
- New `sendio.mailboxes` service (`/mailboxes`): `list`, `create` (`CreateMailboxOptions::address`,
  `::local_part(local_part, domain_id)` or `::free(username)` with `with_org`/`with_display_name`), `update`
  (`UpdateMailboxOptions::new(warmup_enabled)`, `false` is sent), `reset_password` and `delete`. `Mailbox` and
  `ResetMailboxPasswordResponse` carry the IMAP/SMTP `MailboxCredentials`.
- Broadcasts now cover the whole REST API:
  - `CreateBroadcastOptions::draft()` starts an empty draft; `from`, `subject` and the audience are optional
    (`with_from`, `with_subject`, `with_segment_id`). `CreateBroadcastOptions::new(segment_id, from, subject)`
    now sends `segment_id` instead of the deprecated `audience_id`.
  - Create and update take `with_preview_text`, `with_topic_id`, `with_segment_id`, `with_segment_ids`,
    `with_exclude_segment_ids`, `with_audience` (`BroadcastAudience`), `with_unsubscribe_scope`
    (`BroadcastUnsubscribeScope`), `with_tracking` (`false` is sent) and `with_template_engine`
    (`BroadcastTemplateEngine`); update also takes `with_from`/`with_subject` as before.
  - `broadcasts.list` takes `ListBroadcastsOptions` (status, segment_id and search filters plus pagination); a
    `ListOptions` still works.
  - New `broadcasts.estimate`, `broadcasts.test`, `broadcasts.stats`, `broadcasts.list_versions`,
    `broadcasts.get_version`, `broadcasts.update_version` and `broadcasts.restore_version`.
  - `Broadcast` has `segment_id`, `segment_ids`, `exclude_segment_ids`, `audience`, `unsubscribe_scope`,
    `topic_id`, `tracking`, `template_engine`, `version_number`, `failed_reason` and `updated_at`; `name` and
    `audience_id` are now `Option`, so an unnamed draft or an all-contacts broadcast deserializes.
- Fix: `broadcasts.send` no longer puts `broadcast_id` in the request body.

- Webhook signature verification: `sendio::verify_webhook(VerifyWebhookOptions::new(payload, headers, secret))` checks the `svix-id`, `svix-timestamp` and `svix-signature` headers (HMAC-SHA256 over `id.timestamp.payload`, constant-time compare, several `v1` signatures accepted, 5 minute timestamp tolerance configurable with `with_tolerance`). Failures are a typed `WebhookVerificationError`.
- The `axum-verify-event-middleware` example uses `verify_webhook` and answers 400 to a bad signature; the `svix` dev-dependency is gone.
- New dependencies: `base64`, `hmac`, `sha2`.
- User-Agent `sendio-rust:1.1.0`.

## 1.0.0 (2026-10-08)

- First release of the send.io Rust SDK (crate `sendio`).
- Default base URL `https://api.send.io`, overridable with `ConfigBuilder::base_url` or `SENDIO_BASE_URL`.
- `Sendio::default()` reads `SENDIO_API_KEY`.
- User-Agent `sendio-rust:1.0.0`.
