ARG PLATFORM=linux/arm64
FROM --platform=${PLATFORM} golang:1.26.1-trixie AS builder

ARG GOOS=linux
ARG GOARHC=arm64

WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 GOOS=${GOOS} GOARCH=${GOARCH} go build -ldflags="-w -s" -o vpn-killswitch


FROM --platform=${PLATFORM} gcr.io/distroless/static

WORKDIR /app

COPY --from=builder /app/vpn-killswitch .

EXPOSE 8080
ENTRYPOINT ["./vpn-killswitch", "-port", "8080"]
