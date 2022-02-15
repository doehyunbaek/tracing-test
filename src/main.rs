use log::{trace, LevelFilter};
use rustc_serialize::{json, Encodable};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

//structure
//three actors, sender, middle, final
#[tokio::main]
async fn main() {
    //create senders
    println!("starting tracing test");
    json_logger::init("app_name", LevelFilter::Info).unwrap();
    let (middle_tx, mut middle_rx) = mpsc::unbounded_channel::<Message>();
    let (final_tx, mut final_rx) = mpsc::unbounded_channel::<Message>();

    for i in 0..1000 {
        let middle_tx_clone = middle_tx.clone();
        tokio::spawn(async move {
            let temp = Temp {
                a: "job_id".to_string(),
                b: 1,
            };
            let j = serde_json::to_string(&temp).unwrap();
            println!("{}", j);
            // trace!("type: sender, msg: job_{} has begun", i);
            // trace!(
            //     "{}",
            //     json::encode(&LogMessage {
            //         msg: "sample message 2",
            //         event: "structured log"
            //     })
            //     .unwrap()
            // );
            let job_id = format!("{}", i);
            let start_message = Message {
                job_id: job_id.clone(),
                kind: MessageKind::Start,
            };
            middle_tx_clone.send(start_message.clone()).unwrap();
            // trace!("job has begun", {type: "sender", job_id: i});
            // trace!("type: sender, msg:{:?}", start_message);

            let mut counter = 0;
            loop {
                let action_id = Uuid::new_v4().to_string();

                let action_message = Message {
                    job_id: job_id.clone(),
                    kind: MessageKind::Action(action_id.clone()),
                };
                let j = serde_json::to_string(&action_message).unwrap();
                println!("{}", j);
                middle_tx_clone.send(action_message.clone()).unwrap();
                // trace!("type: sender, msg:{:?}", action_message);

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                counter += 1;
                if counter > 100 {
                    trace!("job has finished");
                    break;
                }
            }

            let finish_message = Message {
                job_id: job_id.clone(),
                kind: MessageKind::Finish,
            };
            middle_tx_clone.send(finish_message.clone()).unwrap();
            // trace!("job has begun", {type: 1, job_id: i});
            // trace!("type: sender, msg:{:?}", finish_message);
            let temp = Temp {
                a: "job_id".to_string(),
                b: 1,
            };
            let j = serde_json::to_string(&temp).unwrap();
            println!("{}", j);
        });
    }

    //create middle
    tokio::spawn(async move {
        while let Some(msg) = middle_rx.recv().await {
            // trace!("job has reached middleware", {type: 2, job_id:  });
            // trace!("type: middleware, msg:{:?} ", msg);
            let temp = Temp {
                a: "job_id".to_string(),
                b: 1,
            };
            let j = serde_json::to_string(&temp).unwrap();
            println!("{}", j);
            final_tx.send(msg).unwrap();
        }
    });

    //create final

    while let Some(msg) = final_rx.recv().await {
        trace!("type: final, msg:{:?} ", msg);
    }
}

#[derive(Debug, Clone, Serialize)]
struct Temp {
    a: String,
    b: i64,
}

#[derive(Debug, Clone, Serialize)]
struct Message {
    job_id: String,
    kind: MessageKind,
}

#[derive(Debug, Clone, Serialize)]
enum MessageKind {
    Start,
    Action(String), //action_id
    Finish,
}
