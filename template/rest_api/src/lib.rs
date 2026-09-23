//! REST adapter layer: HTTP handlers (`#[get]`/`#[post]`, `AppJson`, `AppSecurityContext`,
//! `ServiceError` -> `ApiError` mapping) and the authorization chain that protects them.
//! Depends on `stano-starter-rest` (routing, extractors, DI facade) but deliberately
//! **not** `stano-launcher` — this crate is the adapter, not the composition root that
//! assembles and runs the server. Only `launcher/` may depend on `stano-launcher`.

use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use stano_common::ServiceError;
use {{ crate_prefix | replace('-', '_') }}_domain::{Item, ItemId};
use {{ crate_prefix | replace('-', '_') }}_services::ItemService;
use stano_starter_rest::application_context::ApplicationContext;
use stano_starter_rest::security::{AuthorizationBuilder, AuthorizationLayer};
use stano_starter_rest::{
    get, post, ApiError, AppJson, AppPath, AppSecurityContext, Claims, JwtConfig,
};

// ---------------------------------------------------------------------------------------
// Security (app-defined JWT extension type + demo keypair — replace before production use)
// ---------------------------------------------------------------------------------------

/// App-defined JWT extension type (`E` in `Claims<E>`/`SecurityContext<E>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppClaims {
    pub role: String,
}

// Demo-only ES256 (P-256) EC keypair. Never use a hardcoded key beyond local dev/testing
// — generate real keys with `openssl ecparam -name prime256v1 -genkey -noout -out
// private.pem && openssl ec -in private.pem -pubout -out public.pem` and load them from
// the environment (see `.env-example`).
const DEMO_PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgtgbDmCbWzH1rPZlb
qucYzcKQppWx4YxRh0TfnEd0wd6hRANCAATbjOo4G431D+jMHWgoGXaW/vr20Qxn
QuoeHrU++Hh7LgqOwXbpqEmKfJa5Os5GQfdQ579fyDqZ/MepnZz2ijhz
-----END PRIVATE KEY-----";

const DEMO_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE24zqOBuN9Q/ozB1oKBl2lv769tEM
Z0LqHh61Pvh4ey4KjsF26ahJinyWuTrORkH3UOe/X8g6mfzHqZ2c9oo4cw==
-----END PUBLIC KEY-----";

pub fn demo_jwt_config() -> JwtConfig {
    JwtConfig {
        private_key_pem: DEMO_PRIVATE_KEY_PEM.to_string(),
        public_key_pem: DEMO_PUBLIC_KEY_PEM.to_string(),
        expiration_seconds: 3600,
    }
}

/// Test/demo helper: mints a signed token for the given role.
pub fn issue_token(role: &str, jwt_config: &JwtConfig) -> String {
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + jwt_config.expiration_seconds as usize;

    let claims = Claims {
        sub: "demo-user".to_string(),
        session_id: "demo-session".to_string(),
        exp,
        ext: AppClaims {
            role: role.to_string(),
        },
    };
    stano_starter_rest::encode_jwt(&claims, jwt_config).expect("token encodes")
}

/// `/health` and the Swagger UI/OpenAPI doc (dev-only tooling, not app data) are public;
/// `/admin/*` requires the `ADMIN` role; everything else just requires a valid JWT.
pub fn build_authorization(jwt_config: JwtConfig) -> AuthorizationLayer {
    AuthorizationBuilder::<AppClaims>::new()
        .request_matcher("/health")
        .permit_all()
        .request_matcher("/swagger")
        .permit_all()
        // utoipa-swagger-ui redirects "/swagger" -> "/swagger/" and serves its index
        // there; "/swagger/{*rest}" alone doesn't match that (matchit's `{*rest}`
        // wildcard requires a non-empty tail), so the bare trailing-slash path needs
        // its own exact-match rule.
        .request_matcher("/swagger/")
        .permit_all()
        .request_matcher("/swagger/{*rest}")
        .permit_all()
        .request_matcher("/api-docs/{*rest}")
        .permit_all()
        .request_matcher("/admin/{*rest}")
        .has_role(|c: &AppClaims| c.role == "ADMIN")
        .any_request()
        .authenticated()
        .build(jwt_config)
        .expect("authorization chain is valid")
}

// ---------------------------------------------------------------------------------------
// Routes — replace with your own resources, following the same shape.
// ---------------------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct ItemResponse {
    pub id: String,
    pub name: String,
}

impl From<Item> for ItemResponse {
    fn from(item: Item) -> Self {
        Self {
            id: item.id.to_string(),
            name: item.name,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub name: String,
}

#[get(path = "/health", responses((status = 200, body = String)))]
pub async fn health_handler() -> &'static str {
    "ok"
}

#[post(path = "/items", tag = "items", security(("bearerAuth" = [])))]
pub async fn create_item_handler(
    axum::extract::State(ctx): axum::extract::State<Arc<ApplicationContext>>,
    AppJson(req): AppJson<CreateItemRequest>,
) -> AppJson<ItemResponse> {
    let service = ctx.get_trait::<dyn ItemService>();
    AppJson(service.create(req.name).into())
}

#[get(
    path = "/items/{id}",
    tag = "items",
    responses((status = 404, body = String)),
    security(("bearerAuth" = [])),
)]
pub async fn get_item_handler(
    axum::extract::State(ctx): axum::extract::State<Arc<ApplicationContext>>,
    AppPath(id): AppPath<String>,
) -> Result<AppJson<ItemResponse>, ApiError> {
    let item_id = ItemId::from_str(&id)
        .map_err(|_| ApiError::from(ServiceError::InvalidInput("invalid item id".to_string())))?;
    let service = ctx.get_trait::<dyn ItemService>();
    let item = service.get(item_id)?;
    Ok(AppJson(item.into()))
}

#[get(path = "/admin/items", tag = "admin", security(("bearerAuth" = [])))]
pub async fn list_items_admin_handler(
    axum::extract::State(ctx): axum::extract::State<Arc<ApplicationContext>>,
    AppSecurityContext(security_context): AppSecurityContext<AppClaims>,
) -> AppJson<Vec<ItemResponse>> {
    // The caller's role was already enforced by `AuthorizationLayer`; this just proves
    // the claims made it through — replace with real authorization-dependent logic.
    let _ = security_context.ext().role.as_str();
    let service = ctx.get_trait::<dyn ItemService>();
    AppJson(service.list().into_iter().map(Into::into).collect())
}
