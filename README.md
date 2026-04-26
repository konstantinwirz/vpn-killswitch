# vpn-killswitch

## Environment Variables

The following environment variables can be used to configure the application:

- `VPN_PROVIDER_ASN` - Required. ASN (Autonomous System Number) of your VPN provider.
- `VPN_KILLSWITCH_LOG_LEVEL` - Optional. Log level (Error, Warn, Info, Debug). Defaults to "Info".
- `VPN_KILLSWITCH_HTTP_PORT` - Optional. HTTP port for the service to listen on. Defaults to 8080.
