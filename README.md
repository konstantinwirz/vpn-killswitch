# vpn-killswitch

## Environment Variables

The following environment variables can be used to configure the application:

- `IPINFO_API_TOKEN` - Required. API token for the ipinfo.io API.
- `VPN_PROVIDER_ASN` - Required. ASN (Autonomous System Number) of your VPN provider.
- `VPN_KILLSWITCH_LOG_LEVEL` - Optional. Log level (Error, Warn, Info, Debug, Trace). Defaults to "Info".
- `VPN_KILLSWITCH_HTTP_PORT` - Optional. HTTP port for the service to listen on. Defaults to 8080.
