use elasticsearch::{http::transport::Transport, Elasticsearch, SearchParts};
use futures::future::join_all;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = Transport::single_node("https://example.com")?;
    let client = Elasticsearch::new(transport);
    let job_ids = get_unique_job_ids(&client, "tracing").await?;
    let mut futures = vec![];
    for job_id in job_ids {
        futures.push(get_job_data(&client, job_id));
    }
    let results = join_all(futures).await;
    parse_data(results);
    Ok(())
}

async fn get_unique_job_ids(
    client: &Elasticsearch,
    index: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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
    let job_ids = vec![];

    Ok(job_ids)
}

async fn get_job_data(
    client: &Elasticsearch,
    job_id: String,
) -> Result<JobData, Box<dyn std::error::Error>> {
    //todo
    Ok(JobData {})
}

struct JobData {}

/// parse data and produce some meaningful metric and print to console for now
fn parse_data(data: Vec<Result<JobData, Box<dyn std::error::Error>>>) {
    //basic 
}
