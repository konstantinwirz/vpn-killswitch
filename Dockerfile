FROM debian:trixie-slim

WORKDIR /app

COPY vpn-killswitch ./vpn-killswitch

RUN apt update && apt install ca-certificates -y && apt clean && apt autoclean

ENTRYPOINT ["./vpn-killswitch"]
