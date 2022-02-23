 ## Getting started

 You need cargo as a dependency to run this example, you can use docker to run prometheus server or download binary from [prometheus website](https://prometheus.io/download/). This documents supposes you have docker installed.

 To run example client, run

 ```bash
 cargo run --bin tracing-test
 ```

 To run prometheus server, run

 ```bash
 docker run --network host -v $PWD/prometheus.yml:/etc/prometheus/prometheus.yml prom/prometheus
 ```

 Then go to hostname:9090 with your web browser(e.g. 127.0.0.1:9090) and observe middle_actor_latency_bucket value. If you need introduction on using prometheus ui, I recommend watching this great [6 minute video](https://www.youtube.com/watch?v=WUkNnY65htQ&t=204s).
 