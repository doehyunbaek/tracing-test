use std::collections::HashMap;

use elasticsearch::{http::transport::Transport, Elasticsearch, SearchParts};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = Transport::single_node("http://0.0.0.0:9200")?;
    let client = Elasticsearch::new(transport);
    let job_ids = get_unique_job_ids(&client, "tracing").await?;
    let mut futures = vec![];
    for job_id in job_ids {
        futures.push(get_job_data(&client, "tracing", job_id));
    }
    let results = join_all(futures).await;
    parse_data(results);
    Ok(())
}

async fn get_unique_job_ids(
    client: &Elasticsearch,
    index: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    //todo fix query: aggregate unique job_ids
    let response = client
        .search(SearchParts::Index(&[index]))
        .from(0)
        .size(-1) //return all
        .body(json!({
            "aggs": {
                "job_ids": {
                  "terms": { "field": "a.keyword"}
                }
            }
        }))
        .send()
        .await?;

    let mut response_body = response.json::<Value>().await?;
    let buckets = response_body
        .get_mut("aggregations")
        .unwrap()
        .get_mut("job_ids")
        .unwrap()
        .get_mut("buckets")
        .unwrap();
    let mut job_ids = vec![];

    for bucket in buckets.as_array_mut().unwrap().iter_mut() {
        let bucket: Bucket = serde_json::from_value(bucket.take()).unwrap();
        job_ids.push(bucket.key);
    }
    println!("job_ids: {:?}", job_ids);
    Ok(job_ids)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Bucket {
    doc_count: i64,
    key: String,
}

async fn get_job_data(
    client: &Elasticsearch,
    index: &str,
    job_id: String,
) -> Result<(String, Vec<JobDataMessage>), Box<dyn std::error::Error>> {
    //todo send query for getting data
    let response = client
        .search(SearchParts::Index(&[index]))
        .from(0)
        .size(-1) //return all
        .body(json!({
            "query": {
                "match": {
                    "message": "Elasticsearch rust"
                }
            }
        }))
        .send()
        .await?;

    let response_body = response.json::<Value>().await?;
    println!("second query value: {:?}", response_body);
    //todo convert response body to vec[jobdatamessage]

    Ok((job_id, vec![]))
}

///lib으로 넣어서 모듈화 해야되는데.....
#[derive(Debug, Eq)]
struct JobDataMessage {
    job_id: String,
    actor: i64,     //0 for sender, 1 for middle, 2 for final
    action_no: i64, //0 for start, -1 for finish, n for actions
    timestamp: i64, //UNIX timestamp
}

impl PartialEq for JobDataMessage {
    fn eq(&self, other: &Self) -> bool {
        self.actor == other.actor
    }
}

impl PartialOrd for JobDataMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.actor.cmp(&other.actor))
    }
}
impl Ord for JobDataMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.actor.cmp(&other.actor)
    }
}

/// parse data and produce some meaningful metric and print to console for now
/// for now, we only want to see following metrics
/// 1. messages deliver time between sender and middleware
/// 2. messages deliver time between middleware and receiver
fn parse_data(datas: Vec<Result<(String, Vec<JobDataMessage>), Box<dyn std::error::Error>>>) {
    for data in datas {
        let (job_id, data) = data.unwrap();
        let (first_trips, second_trips) = preprocess(&data);
        let first_metric = find_average_and_st(&first_trips);
        let second_metric = find_average_and_st(&second_trips);
        println!(
            "messages deliver time between sender and middleware for job_id: {}, is {:?}",
            job_id, first_metric
        );
        println!(
            "messages deliver time between middleware and receiver for job_id: {}, is {:?}",
            job_id, second_metric
        );
    }
}

fn preprocess<'a>(data: &'a [JobDataMessage]) -> (Vec<i64>, Vec<i64>) {
    //1. first collect by action_id,
    //2. then calculate time between those messages
    let mut sender_to_middle = vec![];
    let mut middle_to_final = vec![];
    let mut sorted_data: HashMap<i64, Vec<&'a JobDataMessage>> = HashMap::new(); //key is action_no, value is [jobdatamessage];
    for a in data {
        match sorted_data.entry(a.action_no) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(a);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(vec![a]);
            }
        }
    }

    for (_action_no, mut messages) in sorted_data {
        if messages.len() != 3 {
            //we should only have messages for sender, middle, final
            continue;
        } else {
            //0 for sender, 1 for middle, 2 for final
            messages.sort();
            sender_to_middle
                .push(messages.get(1).unwrap().timestamp - messages.get(0).unwrap().timestamp);
            middle_to_final
                .push(messages.get(2).unwrap().timestamp - messages.get(1).unwrap().timestamp);
        }
    }

    (sender_to_middle, middle_to_final)
}

fn find_average_and_st(data: &[i64]) -> (f64, f64) {
    let mean = mean(&data).unwrap();
    let std = std_deviation(&data).unwrap();

    (mean, std)
}

fn mean(data: &[i64]) -> Option<f64> {
    let sum = data.iter().sum::<i64>() as f64;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f64),
        _ => None,
    }
}

fn std_deviation(data: &[i64]) -> Option<f64> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f64);

                    diff * diff
                })
                .sum::<f64>()
                / count as f64;

            Some(variance.sqrt())
        }
        _ => None,
    }
}
