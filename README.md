## Goal

Goal of this repo is to see how elaborate elasticsearch query can be for our purpose
we will be running few hundred async jobs simulatenously and sending the logs to the efk

the logs will be one of the kinds below

1. job start
2. action
3. execute
4. job finish

with the fixed format

and for the sake of uniformity, axe-logger rather than env-logger was used

goals we want to achieve are

1. group logs by uuid
2. measure time between each logs from 1
3. measure time between logs that we want from 1
4. export data from 1,2,3 to csv

## Steps to reproduce

Be as specific as possible

only been tested on my local machine with ubuntu 20.04

0. run elasticsearch and kibana locally

```

```

1. run minikube

```
minikube start
```

2. deploy fluentbit

3. deploy our sample application in the minikube

```
kubectl apply -f sample.yaml
```

4. copy and paste queries from the queries file to the kibana dashboard
