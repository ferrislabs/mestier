// `handler.rs` (`WebhookDeliveryHandler`) left with `automation.delivery` and
// `automation.webhook_endpoint` — see migration
// `20260810000004_drop_delivery_and_webhook_endpoints`. The rest of this
// module stays: `SecretCipher`, `signature`, `address_policy` and `resolver`
// are generic building blocks #202's `http.request` connector reuses as-is,
// unrelated to the delivery pipeline that owned this directory before.
pub mod address_policy;
pub mod resolver;
pub mod secret;
pub mod signature;
