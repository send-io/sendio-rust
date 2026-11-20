use std::sync::Arc;

use reqwest::Method;

use crate::{
    Config, Result,
    list_opts::{ListOptions, ListResponse},
    types::Log,
};

/// `Sendio` APIs for `/logs` endpoints.
#[derive(Clone, Debug)]
pub struct LogsSvc(pub(crate) Arc<Config>);

impl LogsSvc {
    /// Retrieve a single API request log.
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn get(&self, log_id: &str) -> Result<Log> {
        let path = format!("/logs/{log_id}");

        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let content = response.json::<Log>().await?;

        Ok(content)
    }

    /// Retrieve a list of API request logs.
    ///
    /// - Default limit: 20
    ///
    /// <https://developer.send.io/docs/api>
    #[maybe_async::maybe_async]
    pub async fn list<T>(&self, list_opts: ListOptions<T>) -> Result<ListResponse<Log>> {
        let request = self.0.build(Method::GET, "/logs").query(&list_opts);
        let response = self.0.send(request).await?;
        let content = response.json::<ListResponse<Log>>().await?;

        Ok(content)
    }
}

#[allow(unreachable_pub)]
pub mod types {
    use std::num::NonZeroU16;

    use serde::{Deserialize, Serialize};

    crate::define_id_type!(LogId);

    #[must_use]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Log {
        pub id: LogId,
        pub created_at: String,
        pub endpoint: String,
        pub method: String,
        pub response_status: NonZeroU16,
        pub user_agent: Option<String>,
        #[serde(default)]
        pub request_body: serde_json::Value,
        #[serde(default)]
        pub response_body: serde_json::Value,
    }
}

#[cfg(test)]
#[allow(clippy::needless_return, clippy::unwrap_used)]
mod test {
    #[cfg(not(feature = "blocking"))]
    use crate::{
        list_opts::ListOptions,
        test::{CLIENT, DebugResult},
    };

    #[tokio_shared_rt::test(shared = true)]
    #[serial_test::serial]
    #[cfg(not(feature = "blocking"))]
    #[ignore = "live API test; run with `cargo test -- --ignored` and SENDIO_API_KEY set"]
    async fn all() -> DebugResult<()> {
        let sendio = &*CLIENT;

        // List
        let logs = sendio.logs.list(ListOptions::default()).await?;
        assert!(!logs.data.is_empty());

        // Get
        let head = logs.data.first().unwrap();

        let _log = sendio.logs.get(&head.id).await?;

        Ok(())
    }
}
