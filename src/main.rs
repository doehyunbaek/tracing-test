use log::{trace, info};
use rand::Rng;
use tokio::sync::mpsc;
use uuid::Uuid;
//structure
//three actors, sender, middle, final
#[tokio::main]
async fn main() {
    env_logger::init();
    let (tx, mut rx) = mpsc::unbounded_channel::<Data>();
    let (tx2, mut rx2) = mpsc::unbounded_channel::<ImportantData>();

    // //for us what defines a span? span should be a job id
    // let span = span!(Level::TRACE, "shaving_yaks");
    // let dd = span.enter();

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
                Data::Tracing(mut data) => {
                    //increase the couter and send the data to final playing actor
                    trace!(
                        "actor: middle, msg: received important data with job: {:?}",
                        data
                    );
                    data.counter += 1;
                    tx2.send(data).unwrap();
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(data) = rx2.recv().await {
            //print out import data
            trace!(
                "actor: final, msg: received important data with job: {:?}",
                data
            );
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
            let important = ImportantData { job_id, counter: 0 };
            tx.send(Data::Tracing(important)).unwrap();
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

#[derive(Debug)]
enum Data {
    Log(UselessData),
    Tracing(ImportantData),
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
