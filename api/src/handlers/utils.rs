use tracing::instrument;

#[instrument]
#[get("/service_version")]
pub async fn service_version() -> String {
    tracing::info!("/service_version");
    std::env::var("SERVICE_VERSION").unwrap_or_else(|_| String::from("unknown"))
}

#[instrument]
#[post("/on-plugin-launched")]
pub async fn on_plugin_launched() {
    tracing::info!("/on-plugin-launched");
}
