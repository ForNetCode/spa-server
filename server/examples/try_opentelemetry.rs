fn main() {
    
}
/*
use std::time::Duration;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{ExportConfig, Protocol, TonicExporterBuilder, WithExportConfig};
use opentelemetry_resource_detectors::{OsResourceDetector, ProcessResourceDetector};
use opentelemetry_sdk::{Resource};
use opentelemetry_sdk::metrics::reader::DefaultAggregationSelector;
use tracing::{Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use opentelemetry_sdk::resource::{ResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector};
use opentelemetry_sdk::trace::config;
use opentelemetry_semantic_conventions::resource;
use tracing_subscriber::EnvFilter;

fn resource() -> Resource{
    let os_resource = OsResourceDetector.detect(Duration::from_secs(0));
    let process_resource = ProcessResourceDetector.detect(Duration::from_secs(0));
    let telemetry_resource = TelemetryResourceDetector.detect(Duration::from_secs(0));
    let sdk_resource = SdkProvidedResourceDetector.detect(Duration::from_secs(0));

    let provided = Resource::new(vec![
        KeyValue::new(resource::SERVICE_NAME, "spa-server"),
        KeyValue::new(resource::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
    ]);

    sdk_resource
        .merge(&provided)
        .merge(&telemetry_resource)
        .merge(&os_resource)
        .merge(&process_resource)
}
fn otlp_exporter() -> TonicExporterBuilder {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        timeout: Duration::from_secs(3),
        protocol: Protocol::Grpc
    };
    opentelemetry_otlp::new_exporter().tonic()
        .with_export_config(export_config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter())
        .with_trace_config(config().with_resource(resource()))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let metrics = opentelemetry_otlp::new_pipeline().metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(otlp_exporter())
        .with_resource(resource())
        .with_aggregation_selector(DefaultAggregationSelector::new()).build()?;

    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())// stdout_layer
        .with(tracing_opentelemetry::MetricsLayer::new(metrics))
        .with(tracing_opentelemetry::layer().with_tracer(tracer)) // traces_layer
        .init();

    Ok(())
}
*/