FROM debian:trixie-slim

WORKDIR /app

COPY vpn-killswitch ./vpn-killswitch

RUN sudo apt update && sudo apt install ca-certificates -y

EXPOSE 8080
ENTRYPOINT ["./vpn-killswitch"]
