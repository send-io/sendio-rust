use axum::{routing::get, Router};
use sendio::{types::CreateEmailBaseOptions, Sendio};
use tower_service::Service;
use worker::*;

fn router() -> Router {
    Router::new().route("/", get(root))
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();

    let sendio = Sendio::new("mx_123456789");

    let from = "Acme <onboarding@yourdomain.com>";
    let to = ["delivered@send.io"];
    let subject = "hello world";
    let html = "<p>it works!</p>";

    let email = CreateEmailBaseOptions::new(from, to, subject).with_html(html);

    let _email = sendio.emails.send(email).await.unwrap();

    Ok(router().call(req).await?)
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
