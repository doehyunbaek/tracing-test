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

```
docker exec -i -t elasticsearch7 cat /usr/share/elasticsearch/config/elasticsearch.yml

docker run -d --link elasticsearch7:elasticsearch -p 5601:5601 --name kibana7 docker.elastic.co/kibana/kibana:7.9.1
```

1. Run minikube

check the minikube default setting

```
minikube start
```

2. Deploy fluentbit

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
