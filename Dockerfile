FROM debian:trixie-slim

WORKDIR /app

COPY vpn-killswitch ./vpn-killswitch

EXPOSE 8080
ENTRYPOINT ["./vpn-killswitch"]
