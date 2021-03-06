## Goal

Goal of this repo is to see how elaborate elasticsearch query can be for our purpose
we will be running few hundred async jobs simulatenously and sending the logs to the efk

The logs will be one of the kinds below

1. job start
2. action
3. job finish

with the fixed format

and for the sake of uniformity, axe-logger rather than env-logger was used

goals we want to achieve are

1. group logs by uuid
2. measure time between each logs from 1
3. measure time between logs that we want from 1
4. export data from 1,2,3 to csv

Once 1,2,3 is done, exporting data so that we can analyze depends on how we use that data, thus manual scripting is necessary. Manual scripting is not the goal of this poc and is left to the team for further work.

## Steps to reproduce

Be as specific as possible

only been tested on my local machine with ubuntu 20.04

0. Run elasticsearch and kibana locally

https://jinhokwon.github.io/devops/elasticsearch/elasticsearch-docker/

```
docker run -d -p 9200:9200 -p 9300:9300 -e "discovery.type=single-node" --name elasticsearch7 docker.elastic.co/elasticsearch/elasticsearch:7.9.1

docker run -d --link elasticsearch7:elasticsearch -p 5601:5601 --name kibana7 docker.elastic.co/kibana/kibana:7.9.1
```

1. Run minikube

check the minikube default setting

```
minikube start
```

2. Deploy fluentbit

since the fluent bit pod running inside the minikube need to access es container outside minikube docker,
you need to change fluent_bit/config.yaml file so that host points to the actual ip address of host.minikube.internal.
For my case, it is 192.168.49.1 but could be different on yours machine

```
kubectl apply -f fluent_bit/role.yaml
kubectl apply -f fluent_bit/config.yaml
kubectl apply -f fluent_bit/daemonset.yaml
```

3. deploy our sample application in the minikube

```
kubectl apply -f sample.yaml
```

4. Copy and paste queries from the queries file to the kibana dashboard

   Queries files are located in the queries/ directory with each number standing for
   group logs by uuid, measure time between each logs from 1, measure time between logs that we want from 1 respectively.

   Queries were written in json format since we will probably use elasticsearch client to make api calls for scripting and in this case, painless scripting is actually very painful.

### Future steps

this may vary based on our need

1. send request to get unique uuids(using script1)
2. send request for each unique uuid to get logs sorted with timestamp(using script2)
3. calculute necessary data(latency etc) based on data from 2
4. add all data from 3?

```rust
//simple pseudocode
struct Log;
struct MeaningfulData;
struct Goal;
fn get_list_of_uuids() -> Vec<String>;
fn send_request_for_uuid(uuid: String) -> Vec<Log>;
fn calculate_data(logs: Vec<Log>) -> MeaningfulData;
fn ananlyze_meaningful_data(data: Vec<MeaningfulData>) -> Goal;

let uuids = get_list_of_uuids();
let mut meaningful_datas= vec![];
for uuid in uuids {
   meanigful_datas.push(calculate_data(send_request_for_uuid(uuid)));
}
let goal = ananlyze_meaningful_data(meaningful_datas);
//done
```

### Demo

0. delete existing index with

```
curl -X DELETE 'http://localhost:9200/tracing'
```

Role of this demo is to show how this poc could function as a tool for analyzing data

1. send query for getting unique job_ids
2. get list of unique job_ids => and compare(maybe make a validator)
3. send request for each job_id and format data
4. calculate data we want and save
5. visualize

```
cargo run --bin demo

```

Then, the result will be both saved as a csv file and printed out on the console.

### Minor debugging

https://serverfault.com/questions/1063166/kube-proxy-wont-start-in-minikube-because-of-permission-denied-issue-with-proc

https://www.elastic.co/guide/en/elasticsearch/reference/6.8/search-request-from-size.html