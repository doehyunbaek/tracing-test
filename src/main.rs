use std::time::SystemTime;

use log::{trace, LevelFilter};
use serde::Serialize;
use tokio::sync::mpsc;

//structure
//three actors, sender, middle, final
#[tokio::main]
async fn main() {
    //create senders
    json_logger::init("app_name", LevelFilter::Info).unwrap();
    let (middle_tx, mut middle_rx) = mpsc::unbounded_channel::<JobData>();
    let (final_tx, mut final_rx) = mpsc::unbounded_channel::<JobData>();

    for i in 0..1 {
        let middle_tx_clone = middle_tx.clone();
        tokio::spawn(async move {
            let job_id = format!("{}", i);
            let job_data = JobData {
                job_id: job_id.clone(),
                actor: 0,
                action_no: 0,
                timestamp: timestamp(),
            };
            let j = serde_json::to_string(&job_data).unwrap();
            print!("{}", j);
            middle_tx_clone.send(job_data).unwrap();
            // trace!("job has begun", {type: "sender", job_id: i});
            // trace!("type: sender, msg:{:?}", start_message);

            let mut counter = 1;
            loop {
                let job_data = JobData {
                    job_id: job_id.clone(),
                    actor: 0,
                    action_no: counter,
                    timestamp: timestamp(),
                };
                let j = serde_json::to_string(&job_data).unwrap();
                println!("{}", j);
                middle_tx_clone.send(job_data).unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                counter += 1;
                if counter > 100 {
                    trace!("job has finished");
                    break;
                }
            }

            let job_data = JobData {
                job_id: job_id.clone(),
                actor: 0,
                action_no: -1,
                timestamp: timestamp(),
            };
            let j = serde_json::to_string(&job_data).unwrap();
            println!("{}", j);
            middle_tx_clone.send(job_data.clone()).unwrap();
        });
    }

    //create middle
    tokio::spawn(async move {
        while let Some(mut msg) = middle_rx.recv().await {
            msg.actor = 1;
            let j = serde_json::to_string(&msg).unwrap();
            println!("{}", j);
            final_tx.send(msg).unwrap();
        }
    });

    //create final

    while let Some(mut msg) = final_rx.recv().await {
        msg.actor = 2;
        let j = serde_json::to_string(&msg).unwrap();
        println!("{}", j);
    }
}

///enum 해야되는데.....
#[derive(Debug, Clone, Serialize)]
struct JobData {
    job_id: String,
    actor: i64,     //0 for sender, 1 for middle, 2 for final
    action_no: i64, //0 for start, -1 for finish, n for actions
    timestamp: i64, //unix timestamp in milliseconds
}

fn timestamp() -> i64 {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time as i64
}
