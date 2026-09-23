use stano_di::environment::{Environment, OsEnvironment};
use stano_launcher::{run, BootstrapConfig, ObservabilityConfig, OtlpProtocol};
use {{ crate_prefix | replace('-', '_') }}_rest_api::{build_authorization, demo_jwt_config};
use utoipa_axum::router::OpenApiRouter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = OsEnvironment::new();

    // Falls back to a demo ES256 keypair so `cargo run` works out of the box locally.
    // Set JWT_PRIVATE_KEY/JWT_PUBLIC_KEY (see .env-example) before deploying anywhere
    // real — never ship the demo keypair.
    let jwt_config = match (env.get("JWT_PRIVATE_KEY"), env.get("JWT_PUBLIC_KEY")) {
        (Some(private_key_pem), Some(public_key_pem)) => stano_security::JwtConfig {
            private_key_pem,
            public_key_pem,
            expiration_seconds: 3600,
        },
        _ => demo_jwt_config(),
    };

    let ctx = {{ project_name | replace('-', '_') }}::build_context();
    let authorization = build_authorization(jwt_config.clone());

    let config = BootstrapConfig {
        port: env
            .get("PORT")
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080),
        jwt_config,
        cors_origins: vec![],
        cors_origin_suffixes: vec![],
        cors_dev_origins: vec![],
        is_dev: env.get("IS_DEV").as_deref() == Some("true"),
        observability: ObservabilityConfig {
            enabled: false,
            protocol: OtlpProtocol::Grpc,
            service_name: "{{ project_name }}".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            resource_attributes: vec![],
            trace_sample_ratio: 1.0,
            log_filter: "info".to_string(),
            metrics_enabled: false,
            prometheus_enabled: false,
            http_logging_enabled: true,
            process_metrics_enabled: false,
        },
        enable_swagger: true,
    };

    run(
        ctx,
        OpenApiRouter::new(),
        config,
        Some(authorization),
        None,
        None,
    )
    .await
}
