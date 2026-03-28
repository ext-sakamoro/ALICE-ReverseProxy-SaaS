#![allow(dead_code)]
use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// ── State ───────────────────────────────────────────────────
struct AppState {
    start_time: Instant,
    stats: Mutex<Stats>,
}

struct Stats {
    total_routes: u64,
    total_upstreams: u64,
    total_circuit_trips: u64,
    total_health_checks: u64,
    requests_proxied: u64,
}

// ── Types ───────────────────────────────────────────────────
#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_ops: u64 }

// Routes
#[derive(Deserialize)]
#[allow(dead_code)]
struct RouteRequest {
    host: Option<String>,
    path_prefix: Option<String>,
    upstream: Option<String>,
    strip_prefix: Option<bool>,
    timeout_ms: Option<u64>,
}
#[derive(Serialize)]
struct RouteResponse {
    route_id: String,
    status: String,
    host: String,
    path_prefix: String,
    upstream: String,
    strip_prefix: bool,
    timeout_ms: u64,
    elapsed_us: u128,
}

// Upstream
#[derive(Deserialize)]
#[allow(dead_code)]
struct UpstreamRequest {
    name: Option<String>,
    targets: Option<Vec<String>>,
    strategy: Option<String>,
    max_conns: Option<u32>,
}
#[derive(Serialize)]
struct UpstreamResponse {
    upstream_id: String,
    status: String,
    name: String,
    target_count: usize,
    strategy: String,
    max_conns: u32,
    elapsed_us: u128,
}

// Circuit Breaker
#[derive(Deserialize)]
#[allow(dead_code)]
struct CircuitBreakerRequest {
    upstream: Option<String>,
    threshold: Option<u32>,
    timeout_secs: Option<u64>,
    half_open_requests: Option<u32>,
}
#[derive(Serialize)]
struct CircuitBreakerResponse {
    cb_id: String,
    status: String,
    upstream: String,
    state: String,
    threshold: u32,
    timeout_secs: u64,
    half_open_requests: u32,
    elapsed_us: u128,
}

// Health Check
#[derive(Deserialize)]
#[allow(dead_code)]
struct HealthCheckRequest {
    upstream: Option<String>,
    path: Option<String>,
    interval_secs: Option<u64>,
}
#[derive(Serialize)]
struct HealthCheckResponse {
    check_id: String,
    upstream: String,
    path: String,
    healthy_targets: u32,
    unhealthy_targets: u32,
    interval_secs: u64,
    elapsed_us: u128,
}

// Stats
#[derive(Serialize)]
struct StatsResponse {
    total_routes: u64,
    total_upstreams: u64,
    total_circuit_trips: u64,
    total_health_checks: u64,
    requests_proxied: u64,
}

// ── Main ────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "reverseproxy_engine=info".into()))
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_routes: 0, total_upstreams: 0, total_circuit_trips: 0,
            total_health_checks: 0, requests_proxied: 0,
        }),
    });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/proxy/routes", post(routes))
        .route("/api/v1/proxy/upstream", post(upstream))
        .route("/api/v1/proxy/circuit-breaker", post(circuit_breaker))
        .route("/api/v1/proxy/health-check", post(health_check))
        .route("/api/v1/proxy/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("PROXY_ADDR").unwrap_or_else(|_| "0.0.0.0:8130".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("ReverseProxy Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ────────────────────────────────────────────────
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health {
        status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(),
        total_ops: st.total_routes + st.total_upstreams + st.total_health_checks,
    })
}

async fn routes(State(s): State<Arc<AppState>>, Json(req): Json<RouteRequest>) -> Json<RouteResponse> {
    let t = Instant::now();
    let host = req.host.unwrap_or_else(|| "example.com".into());
    let path_prefix = req.path_prefix.unwrap_or_else(|| "/".into());
    let upstream = req.upstream.unwrap_or_else(|| "default-pool".into());
    let strip_prefix = req.strip_prefix.unwrap_or(false);
    let timeout_ms = req.timeout_ms.unwrap_or(30_000);
    s.stats.lock().unwrap().total_routes += 1;
    Json(RouteResponse {
        route_id: uuid::Uuid::new_v4().to_string(),
        status: "registered".into(),
        host, path_prefix, upstream, strip_prefix, timeout_ms,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn upstream(State(s): State<Arc<AppState>>, Json(req): Json<UpstreamRequest>) -> Json<UpstreamResponse> {
    let t = Instant::now();
    let name = req.name.unwrap_or_else(|| "default-pool".into());
    let targets = req.targets.unwrap_or_else(|| vec!["http://127.0.0.1:8081".into()]);
    let strategy = req.strategy.unwrap_or_else(|| "round-robin".into());
    let max_conns = req.max_conns.unwrap_or(100);
    let target_count = targets.len();
    s.stats.lock().unwrap().total_upstreams += 1;
    Json(UpstreamResponse {
        upstream_id: uuid::Uuid::new_v4().to_string(),
        status: "configured".into(),
        name, target_count, strategy, max_conns,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn circuit_breaker(State(s): State<Arc<AppState>>, Json(req): Json<CircuitBreakerRequest>) -> Json<CircuitBreakerResponse> {
    let t = Instant::now();
    let upstream = req.upstream.unwrap_or_else(|| "default-pool".into());
    let threshold = req.threshold.unwrap_or(5);
    let timeout_secs = req.timeout_secs.unwrap_or(30);
    let half_open_requests = req.half_open_requests.unwrap_or(3);
    s.stats.lock().unwrap().total_circuit_trips += 1;
    Json(CircuitBreakerResponse {
        cb_id: uuid::Uuid::new_v4().to_string(),
        status: "configured".into(),
        upstream, state: "closed".into(),
        threshold, timeout_secs, half_open_requests,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn health_check(State(s): State<Arc<AppState>>, Json(req): Json<HealthCheckRequest>) -> Json<HealthCheckResponse> {
    let t = Instant::now();
    let upstream = req.upstream.unwrap_or_else(|| "default-pool".into());
    let path = req.path.unwrap_or_else(|| "/health".into());
    let interval_secs = req.interval_secs.unwrap_or(10);
    s.stats.lock().unwrap().total_health_checks += 1;
    Json(HealthCheckResponse {
        check_id: uuid::Uuid::new_v4().to_string(),
        upstream, path,
        healthy_targets: 2, unhealthy_targets: 0,
        interval_secs,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse {
        total_routes: st.total_routes,
        total_upstreams: st.total_upstreams,
        total_circuit_trips: st.total_circuit_trips,
        total_health_checks: st.total_health_checks,
        requests_proxied: st.requests_proxied,
    })
}
