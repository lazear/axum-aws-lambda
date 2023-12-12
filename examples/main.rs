use axum::extract::State;
use axum::http::header::{ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN};
use axum::response::IntoResponse;
use axum::{
    routing::{get, post},
    Json, Router,
};
use hyper::Request;
use hyper::{Body, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

#[derive(Default)]
struct AppState {
    data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Data {
    name: String,
}

async fn post_data(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<Data>,
) -> impl IntoResponse {
    let mut state = state.lock().unwrap();
    state.data.push(payload);
    StatusCode::CREATED
}

async fn get_data(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().unwrap();
    (StatusCode::OK, Json(state.data.clone()))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    let state = Arc::new(Mutex::new(AppState::default()));

    // Trace every request
    let trace_layer =
        TraceLayer::new_for_http().on_request(|_: &Request<Body>, _: &tracing::Span| {
            tracing::info!(message = "begin request!")
        });

    // Set up CORS
    let cors_layer = CorsLayer::new()
        .allow_headers([ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN])
        .allow_methods(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    // Wrap an `axum::Router` with our state, CORS, Tracing, & Compression layers
    let app = Router::new()
        .route("/", post(post_data))
        .route("/", get(get_data))
        .layer(cors_layer)
        .layer(trace_layer)
        .layer(CompressionLayer::new().gzip(true).deflate(true))
        .with_state(state);

    #[cfg(debug_assertions)]
    {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    // If we compile in release mode, use the Lambda Runtime
    #[cfg(not(debug_assertions))]
    {
        // To run with AWS Lambda runtime, wrap in our `LambdaLayer`
        let app = tower::ServiceBuilder::new()
            .layer(axum_aws_lambda::LambdaLayer::default())
            .service(app);

        lambda_http::run(app).await.unwrap();
    }
}
