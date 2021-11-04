# Hybrid Traffic Mesh Proxy

[![license](https://img.shields.io/github/license/mashape/apistatus.svg)](https://github.com/i-pva/stopless/blob/master/LICENSE)

L7 proxy on kubernetes

dependencies: 
- routeagent: refresh proxy routes fetched with k8s sdk

## register routes 
```shell script
curl -v  --unix-socket /tmp/hpx/hpx.sock  --request POST 'http://unix/route/register' \
--header 'Content-Type: application/json' \
--data-raw '
[
  {
    "servant": "testsvc",
    "routes": [
      {
        "path": "/testsvc/v1/test1",
        "kind": "fuzzy"
      }
    ],
    "endpoints": [
      "127.0.0.1:9098",
      "127.0.0.1:9090"
    ]
  },
  {
    "servant": "testsvc2",
    "routes": [
      {
        "path": "/testsvc2/v1/test2",
        "kind": "fuzzy"
      }
    ],
    "endpoints": [
      "127.0.0.1:9099",
      "127.0.0.1:9091"
    ]
  }
]
```

## Configuration

```shell script
OPEN_TRACING=127.0.0.1:6831 #jaeger 
SAMPLING_PERCENTAGE=10  #sampling percentage 0-100
CONNECT_TIMEOUT = 10  # forward client socket connect timeout
KEEPALIVE_TIMEOUT =20 # client keep alive timeout
```
