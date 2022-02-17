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
    // futures.push(get_job_data(&client, "tracing", job_ids.get(0).unwrap().to_string()));

    for job_id in job_ids {
        futures.push(get_job_data(&client, "tracing", job_id));
    }
    let results = join_all(futures).await;
    // println!("results: {:?}", results);
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
        .size(1) //todo -1 is default which is 10
        .body(json!({
            "aggs": {
                "job_ids": {
                  "terms": { "field": "job_id.keyword"}
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
    // println!("first query response: {}", buckets);
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
    //todo fix query, change the required field + term name and value as real job_id
    println!("this is job_id: {}", job_id);
    let query = json!({
      "_source": ["job_id", "actor", "action_no", "timestamp"],
      "query": {
        "term": {
          "job_id": {
            "value": job_id
          }
        }
      }
    });
    let response = client
        .search(SearchParts::Index(&[index]))
        .from(0)
        .size(100) //todo need to check what the max would be for this value since we would like to return all data in certain periods
        .body(query)
        .send()
        .await?;
    let mut response_body = response.json::<Value>().await?;
    let data = response_body
        .get_mut("hits")
        .unwrap()
        .get_mut("hits")
        .unwrap();
    // println!("second query value: {:?}", data);
    let mut job_data = vec![];
    for data in data.as_array_mut().unwrap().iter_mut() {
        let result: Result<Hit, serde_json::Error> = serde_json::from_value(data.take());
        println!("dd: {:?}", result);
        if let Ok(data) = result {
            job_data.push(data._source);
        }
    }

    //todo convert response body to vec[jobdatamessage]

    Ok((job_id, job_data))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Hit {
    _index: String,
    _type: String,
    _id: String,
    _score: f64,
    _source: JobDataMessage,
}

///lib으로 넣어서 모듈화 해야되는데.....
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
struct JobDataMessage {
    job_id: String, //pretty much for debugging purpose
    actor: i64,     //0 for sender, 1 for middle, 2 for final
    action_no: i64, //0 for start, -1 for finish, n for actions
    // #[serde(rename = "@timestamp")]
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
        // println!("job_id: {}, and data: {:?}", job_id, data);
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
            // println!("incomplete messages: {:?}", messages);
            continue;
        } else {
            //0 for sender, 1 for middle, 2 for final
            // println!("complete messages: {:?}", messages);
            messages.sort();
            // println!(
            //     "first metric: {}",
            //     messages.get(1).unwrap().timestamp - messages.get(0).unwrap().timestamp
            // );
            // println!(
            //     "second metric: {}",
            //     messages.get(2).unwrap().timestamp - messages.get(1).unwrap().timestamp
            // );
            sender_to_middle
                .push(messages.get(1).unwrap().timestamp - messages.get(0).unwrap().timestamp);
            middle_to_final
                .push(messages.get(2).unwrap().timestamp - messages.get(1).unwrap().timestamp);
        }
    }

    // println!(
    //     "processed data: {:?}, {:?}",
    //     sender_to_middle, middle_to_final
    // );
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
