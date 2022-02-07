use rand::Rng;
use tokio::sync::mpsc;
use tracing::{info, span, trace, Level, Span};
use uuid::Uuid;
//structure
//three actors, sender, middle, final
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (tx, mut rx) = mpsc::unbounded_channel::<Data>();
    let (tx2, mut rx2) = mpsc::unbounded_channel::<(ImportantData, Span)>();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Data::Log(data) => {
                    //just print this out with log
                    info!(
                        "actor: middle, msg: received useless data with job: {:?}",
                        data
                    );
                }
                Data::Tracing(mut data, span) => {
                    //increase the couter and send the data to final playing actor
                    trace!(
                        "actor: middle, msg: received important data with job: {:?}, context: {:?}",
                        data,
                        span.metadata().unwrap().fields()
                    );
                    data.counter += 1;
                    tx2.send((data, span)).unwrap();
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some((data, mut span)) = rx2.recv().await {
            //print out import data
            trace!(
                "actor: final, msg: received important data with job: {:?}, context: {:?}",
                data,
                span.metadata().unwrap().fields()
            );
            // span.drop();
        }
    });

    let mut rng = rand::thread_rng();

    //keep producing msgs to the receivng actor
    loop {
        let ran_number: i32 = rng.gen_range(0..10);
        if ran_number < 3 {
            let job_id = Uuid::new_v4().to_string();
            let useless = UselessData { job_id };
            tx.send(Data::Log(useless)).unwrap();
        } else {
            //important data
            let job_id = Uuid::new_v4().to_string();
            let span = span!(Level::TRACE, "static string"); //dynamic span id is not recommended. key value pair is the recommende way
                                                             // let _ = span.enter();
            let important = ImportantData { job_id, counter: 0 };
            tx.send(Data::Tracing(important, span)).unwrap();
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

#[derive(Debug)]
enum Data {
    Log(UselessData),
    Tracing((ImportantData), Span),
}

#[derive(Debug)]
struct UselessData {
    pub job_id: String,
}

#[derive(Debug)]
struct ImportantData {
    pub job_id: String,
    pub counter: u8,
}
