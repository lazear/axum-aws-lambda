use std::{future::Future, marker::PhantomData, pin::Pin};

use axum::{
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use lambda_http::Service;
use serde::{Deserialize, Serialize};
use tower::{Layer, ServiceBuilder};

#[derive(Deserialize, Serialize, Debug)]
struct Data {
    name: String,
}

async fn data(Json(payload): Json<Data>) -> impl IntoResponse {
    let new = Data {
        name: format!("Hello, {}", payload.name),
    };
    (StatusCode::CREATED, Json(new))
}

pub struct LambdaToAxum<'a, S> {
    service: S,
    _phantom: PhantomData<&'a S>,
}

impl<'a, S> Service<Request<lambda_http::Body>> for LambdaToAxum<'a, S>
where
    S: Service<Request<axum::body::Body>>,
    S::Future: 'a,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'a>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<lambda_http::Body>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let body = match body {
            lambda_http::Body::Empty => axum::body::Body::default(),
            lambda_http::Body::Text(t) => t.into(),
            lambda_http::Body::Binary(v) => v.into(),
        };
        let request = Request::from_parts(parts, body);
        Box::pin(self.service.call(request))
    }
}

pub struct AxumToLambda<'a, S> {
    service: S,
    _phantom: PhantomData<&'a S>,
}

impl<'a, S, R> Service<R> for AxumToLambda<'a, S>
where
    S: Service<R>,
    S::Response: IntoResponse,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: 'a,
{
    type Response = Response<lambda_http::Body>;
    type Error = lambda_http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'a>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: R) -> Self::Future {
        let fut = self.service.call(req);
        let fut = async move {
            let (parts, body) = fut.await?.into_response().into_parts();
            let bytes = hyper::body::to_bytes(body).await?;
            let bytes: &[u8] = &bytes;
            let resp = match std::str::from_utf8(bytes) {
                Ok(s) => Response::from_parts(parts, s.into()),
                Err(_) => Response::from_parts(parts, bytes.into()),
            };
            Ok(resp)
        };
        Box::pin(fut)
    }
}

#[derive(Default)]
pub struct InnerLayer<'a> {
    _phantom: PhantomData<&'a ()>,
}


#[derive(Default)]
pub struct OuterLayer<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a, S> Layer<S> for InnerLayer<'a>
where
    S: 'a,
{
    type Service = AxumToLambda<'a, S>;

    fn layer(&self, inner: S) -> Self::Service {
        AxumToLambda {
            service: inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, S> Layer<S> for OuterLayer<'a>
where
    S: 'a,
{
    type Service = LambdaToAxum<'a, S>;

    fn layer(&self, inner: S) -> Self::Service {
        LambdaToAxum {
            service: inner,
            _phantom: PhantomData,
        }
    }
}

#[tokio::main]
async fn main() {
    let app: Router = Router::new().route("/data", post(data));

    let service = ServiceBuilder::new()
        .layer(OuterLayer::default())
        .layer(InnerLayer::default())
        .service(app);

    lambda_http::run(service).await.unwrap();
}
