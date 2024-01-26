use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Response,
};
use once_cell::sync::Lazy;
use prometheus::{gather, register_int_gauge, Encoder, IntGauge, TextEncoder};
use std::convert::Infallible;
use tracing::error;

pub static CHECKED_COUNT: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("checked_count", " ").unwrap());
pub static ERROR_COUNT: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("error_count", " ").unwrap());
pub static MISMATCH_COUNT: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("mismatch_count", " ").unwrap());

pub async fn serve_metrics(port: u16) {
    let make_service = make_service_fn(|_| async {
        Ok::<_, Infallible>(service_fn(|_req| async {
            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            let metric_families = gather();
            encoder.encode(&metric_families, &mut buffer).unwrap();

            Ok::<_, Infallible>(Response::new(Body::from(buffer)))
        }))
    });

    let server = Server::bind(&([0, 0, 0, 0], port).into()).serve(make_service);

    if let Err(e) = server.await {
        error!("[Server] Exit with error: {}", e);
    }
}
