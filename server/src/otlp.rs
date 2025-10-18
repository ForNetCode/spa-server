use opentelemetry::KeyValue;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::trace::config;
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use opentelemetry_semantic_conventions::trace::SERVICE_VERSION;
use tracing_core::Level;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_otlp(endpoint: String) -> anyhow::Result<()> {
    let resource = Resource::new(vec![
        KeyValue::new(SERVICE_NAME, "spa-server"),
        KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
    ]);
    let mut export_config = ExportConfig::default();
    export_config.endpoint = endpoint;
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_export_config(export_config);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(config().with_resource(resource))
        .install_batch(runtime::Tokio)?;

    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .with(OpenTelemetryLayer::new(tracer))
        .init();
    Ok(())
}
