//! OpenTelemetry initialisation for Ferrisletter.
//!
//! This module is only compiled when the `telemetry` Cargo feature is enabled.
//! Call [`init`] to set up the OTLP trace exporter and get a [`tracing`] layer
//! that bridges spans to OpenTelemetry. Combine it with the fmt layer in a
//! [`tracing_subscriber::Registry`].

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::Layer;

use crate::config::TelemetryConfig;

/// Initialise the OpenTelemetry pipeline and return a tracing layer.
///
/// The returned layer is boxed so it can compose with any subscriber stack.
/// Add it to a [`tracing_subscriber::Registry`] alongside the fmt layer.
/// Call [`shutdown`] before the process exits to flush remaining spans.
pub fn init<S>(
    config: &TelemetryConfig,
) -> Result<Box<dyn Layer<S> + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>
where
    S: tracing::Subscriber
        + for<'span> tracing_subscriber::registry::LookupSpan<'span>
        + Send
        + Sync,
{
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.endpoint)
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name(config.service_name.clone())
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer("ferrisletter");

    // Store the provider globally so we can shut it down later.
    // In OTel 0.31+, shutdown is called on the provider directly.
    PROVIDER.set(provider).ok();

    Ok(Box::new(OpenTelemetryLayer::new(tracer)))
}

/// Global provider handle for shutdown.
static PROVIDER: std::sync::OnceLock<SdkTracerProvider> = std::sync::OnceLock::new();

/// Gracefully flush and shut down the OpenTelemetry pipeline.
pub fn shutdown() {
    if let Some(provider) = PROVIDER.get()
        && let Err(e) = provider.shutdown()
    {
        eprintln!("OpenTelemetry shutdown error: {e}");
    }
}
