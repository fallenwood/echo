use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{SpanExporter, WithExportConfig, LogExporter};
use opentelemetry_sdk::{
  logs::SdkLoggerProvider,
  propagation::TraceContextPropagator,
  trace::{SdkTracerProvider, TracerProviderBuilder}, Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracer(service_name: String, endpoint: String) -> SdkTracerProvider {
  global::set_text_map_propagator(TraceContextPropagator::new());

  let exporter = SpanExporter::builder()
    .with_tonic()
    .with_endpoint(endpoint)
    .build()
    .unwrap();

  let provider = TracerProviderBuilder::default()
    .with_simple_exporter(exporter)
    .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
    .with_resource(
      Resource::builder()
        .with_service_name(service_name)
        .build(),
    )
    .build();

  global::set_tracer_provider(provider.clone());
  provider
}

pub fn init_logs(service_name: String, endpoint: String) -> SdkLoggerProvider {
  let exporter = LogExporter::builder()
    .with_tonic()
    .with_endpoint(endpoint)
    .build()
    .unwrap();

  let logger_provider = SdkLoggerProvider::builder()
    .with_simple_exporter(exporter)
    .with_simple_exporter(opentelemetry_stdout::LogExporter::default())
    .with_resource(
      Resource::builder()
        .with_service_name(service_name)
        .build(),
    )
    .build();
  let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);
  tracing_subscriber::registry().with(otel_layer).init();

  logger_provider
}
