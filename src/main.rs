use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

use tokio::sync::mpsc;

use prometheus_client::encoding::text::{encode, Encode};
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;

#[tokio::main]
async fn main() {
    let mut registry = Registry::default();
    let middle_actor_latency = Family::<Labels, Histogram>::new_with_constructor(|| {
        Histogram::new(exponential_buckets(2.0, 2.0, 7))
    });
    registry.register(
        "middle_actor_latency",
        "latency for middle actor",
        middle_actor_latency.clone(),
    );

    let mut app = tide::with_state(State {
        registry: Arc::new(Mutex::new(registry)),
    });

    // setup metric endpoint serving. I used tide here as original example
    // used tide, but it does not matter which server framework is used, if it can
    // get access to histogram registry, it will do
    setup_serve(&mut app);

    let (middle_tx, mut middle_rx) = mpsc::unbounded_channel::<i64>();

    // we send data to our middle_actor here.
    let middle_tx_clone = middle_tx.clone();
    tokio::spawn(async move {
        loop {
            // throttle sending a little bit
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            middle_tx_clone.send(timestamp()).unwrap();
        }
    });

    // we receive data at middle actor and measure latency here.
    let middle_actor_latency_clone = middle_actor_latency.clone();
    tokio::spawn(async move {
        while let Some(msg) = middle_rx.recv().await {
            // we mock actor activity with random sleep
            let mut rng: StdRng = SeedableRng::seed_from_u64(0);
            let randval = rng.gen_range(0..1280);
            tokio::time::sleep(tokio::time::Duration::from_micros(randval)).await;

            // measure latency with timestamp difference
            let timediff = timestamp() - msg;
            middle_actor_latency_clone
                .get_or_create(&Labels {
                    actor: "middle".to_owned(),
                })
                .observe(timediff as f64);
        }
    });

    let _ = app.listen("0.0.0.0:8080").await;
}

fn timestamp() -> i64 {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time as i64
}

#[derive(Clone)]
struct State {
    registry: Arc<Mutex<Registry<Family<Labels, Histogram>>>>,
}

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
struct Labels {
    actor: String,
}

fn setup_serve(app: &mut tide::Server<State>) {
    tide::log::start();
    app.at("/metrics")
        .get(|req: tide::Request<State>| async move {
            let mut encoded = Vec::new();
            encode(&mut encoded, &req.state().registry.lock().unwrap()).unwrap();
            let response = tide::Response::builder(200)
                .body(encoded)
                .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
                .build();
            Ok(response)
        });
}
