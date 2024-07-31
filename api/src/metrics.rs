use crate::errors::CoreError;
use prometheus::core::{AtomicU64, GenericCounter, GenericCounterVec};
use prometheus::{Encoder, IntCounter, IntCounterVec, Opts, Registry, TextEncoder};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, State};
use tracing::instrument;

const NAMESPACE: &str = "zksync_api";

#[derive(Clone)]
pub(crate) struct Metrics {
    pub num_distinct_users: GenericCounterVec<AtomicU64>,
    pub num_plugin_launches: GenericCounter<AtomicU64>,
    pub num_of_compilations: GenericCounter<AtomicU64>,
}

#[rocket::async_trait]
impl Fairing for Metrics {
    fn info(&self) -> Info {
        Info {
            name: "Metrics fairing",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        req.client_ip().map(|val| {
            self.num_distinct_users
                .with_label_values(&[val.to_string().as_str()])
                .inc()
        });

        match req.uri().path().as_str() {
            "/compile" | "/compile-async" => self.num_of_compilations.inc(),
            "/on-plugin-launched" => self.num_plugin_launches.inc(),
            _ => {}
        }
    }
}

// TODO: Result<Registry>
pub(crate) fn create_metrics(registry: Registry) -> Result<Metrics, CoreError> {
    let opts = Opts::new("num_distinct_users", "Number of distinct users").namespace(NAMESPACE);
    let num_distinct_users = IntCounterVec::new(opts, &["ip"])?;
    registry.register(Box::new(num_distinct_users.clone()))?;

    let opts = Opts::new("num_plugin_launches", "Number of plugin launches").namespace(NAMESPACE);
    let num_plugin_launches = IntCounter::with_opts(opts)?;
    registry.register(Box::new(num_plugin_launches.clone()))?;

    let opts = Opts::new("num_of_compilations", "Number of compilation runs").namespace(NAMESPACE);
    let num_of_compilations = IntCounter::with_opts(opts)?;
    registry.register(Box::new(num_of_compilations.clone()))?;

    Ok(Metrics {
        num_distinct_users,
        num_plugin_launches,
        num_of_compilations,
    })
}

#[instrument]
#[get("/metrics")]
pub(crate) async fn metrics(registry: &State<Registry>) -> String {
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => match String::from_utf8(buffer) {
            Ok(val) => val,
            Err(_) => "Non utf8 metrics".into(),
        },
        Err(_) => "Encode error".into(),
    }
}
