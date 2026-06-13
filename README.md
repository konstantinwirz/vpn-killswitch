# vpn-killswitch


Starts application on port `8080` with a polling interval of 20 seconds. Provides a http endpoint `localhost:8080/` which responds with `200` if the ASN of the current internet tporvider is `3320`, otherwise responds with `503`.
```shell
vpn-killswitch --port 8080 --poll-interval 20 --log-level debug --asn 3320
```
