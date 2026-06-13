# vpn-killswitch
[![ci](https://github.com/konstantinwirz/vpn-killswitch/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/konstantinwirz/vpn-killswitch/actions/workflows/ci.yaml)

Starts application on port `8080` with a polling interval of 20 seconds. Provides a http endpoint `localhost:8080/killswitch` which responds with `200` if the ASN of the current internet provider is `3320`, otherwise responds with `503`.
```shell
vpn-killswitch --port 8080 --poll-interval 20 --log-level debug --asn 3320
```
