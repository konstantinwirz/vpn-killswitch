package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"math/rand"
	"net"
	"net/http"
	"net/url"
	"strings"
)

// PublicIPLookupResponse represents a response from a Public IP provider
type PublicIPLookupResponse struct {
	IP  net.IP
	ASN string
}

var ErrInvalidIP error = errors.New("Invalid IP address")
var ErrInvalidASN error = errors.New("Invalid ASN")
var ErrNoProviderReachable error = errors.New("No Public IP Provider is reachable")

func ParsePublicIPLookupResponse(ip, asn string) (*PublicIPLookupResponse, error) {
	ipV6 := net.ParseIP(ip)
	if ipV6 == nil {
		return nil, ErrInvalidIP
	}

	ipV4 := ipV6.To4()
	if ipV4 == nil {
		return nil, ErrInvalidIP
	}

	if len(asn) < 3 || strings.ToUpper(asn[:2]) != "AS" {
		return nil, ErrInvalidASN
	}

	return &PublicIPLookupResponse{IP: ipV4, ASN: strings.ToUpper(asn)}, nil
}

// String confroms to Stringer protocol
func (p PublicIPLookupResponse) String() string {
	return fmt.Sprintf("PublicIPLookupResponse [ IP = %s, ASN = %s ]", p.IP, p.ASN)
}

// PublicIPProvider is a generic provider of Public IPs and ASNs
type PublicIPProvider[T any] interface {
	// URL provides the URL to call
	URL() *url.URL
	// AdditionalHeaders can provider additional headers if needed, if not neccessary - let empty
	AdditionalHeaders() map[string]string
	// Transform transforms a spcecific public ip response to the PublicIPLookupResponse
	Transform(specificResponse T) (*PublicIPLookupResponse, error)
}

// Lookup makes a public ip lookup using given provider
func Lookup[T any](provider PublicIPProvider[T]) (*PublicIPLookupResponse, error) {
	logger := slog.Default().With("func", "Lookup", "provider", provider.URL().String())
	logger.Debug("about to make a public ip lookup")

	client := http.Client{}
	req, err := http.NewRequest("GET", provider.URL().String(), nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create GET request for the url '%s'", provider.URL().String())
	}

	req.Header.Add("User-Agent", "vpn-killswitch/0.1.0")
	req.Header.Add("Accept", "application/json")
	for k, v := range provider.AdditionalHeaders() {
		req.Header.Add(k, v)
	}

	logger.Debug("Added headers", "headers", req.Header)

	resp, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to make GET request to %s: %w", provider.URL(), err)
	}

	logger.Debug("got response, trying to unmarshal", "status_code", resp.StatusCode)

	b, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	var specificPublicIpResponse T
	if err := json.Unmarshal(b, &specificPublicIpResponse); err != nil {
		return nil, fmt.Errorf("failed to unmarschal specific response: %w", err)
	}

	logger.Debug("unmarschalled response, transforming it", "response", specificPublicIpResponse)

	publicIPResponse, err := provider.Transform(specificPublicIpResponse)
	if err != nil {
		return nil, fmt.Errorf("failed to transform specific lookup response: %w", err)
	}

	logger.Info("got public ip response", "response", publicIPResponse)

	return publicIPResponse, nil
}

var lookups = []func() (*PublicIPLookupResponse, error){
	func() (*PublicIPLookupResponse, error) { return Lookup(IfConfigCo) },
	func() (*PublicIPLookupResponse, error) { return Lookup(IpApiCo) },
	func() (*PublicIPLookupResponse, error) { return Lookup(IpApiCom) },
	func() (*PublicIPLookupResponse, error) { return Lookup(IpWhoIs) },
	func() (*PublicIPLookupResponse, error) { return Lookup(MyIpIo) },
}

func LookupWithRandomProvider(maxAttempts int) (*PublicIPLookupResponse, error) {
	logger := slog.Default().With("maxAttempts", maxAttempts)
	for attempt := 0; attempt < maxAttempts; attempt++ {
		logger.Debug("about to make a public ip lookup using a random provider", "attempt", attempt+1)
		resp, err := lookups[rand.Intn(len(lookups))]()
		if err != nil {
			logger.Warn("failed to get a public ip reponse", "attempt", attempt+1)
			continue
		}

		logger.Debug("got public ip", "attempt", attempt+1, "resp", resp)
		return resp, nil
	}

	return nil, ErrNoProviderReachable
}

// genericPublicIPLookupResponse represents a generic response since
// in 90% of cases the fields called like this
type genericPublicIPLookupResponse struct {
	IP  string `json:"ip"`
	ASN string `json:"asn"`
}

func (r genericPublicIPLookupResponse) transform() (*PublicIPLookupResponse, error) {
	return ParsePublicIPLookupResponse(r.IP, r.ASN)
}

// mustParseURL converts given raw url string to a URL instance
// panics if the url is not wellformed
func mustParseURL(rawURL string) *url.URL {
	url, err := url.Parse(rawURL)
	if err != nil {
		panic(err)
	}

	return url
}

/*
ifConfigCo represents ifconfig.co.

rate limit: 1 req/minute

test it:

curl -H "Accept: application/json" -H "User-Agent: vpn-killswitch/0.1.0" https://ifconfig.co/
*/
type ifConfigCo struct{}

var IfConfigCo = ifConfigCo{}

var _ PublicIPProvider[genericPublicIPLookupResponse] = ifConfigCo{}

func (p ifConfigCo) URL() *url.URL                        { return mustParseURL("https://ifconfig.co/") }
func (p ifConfigCo) AdditionalHeaders() map[string]string { return map[string]string{} }
func (p ifConfigCo) Transform(specificResponse genericPublicIPLookupResponse) (*PublicIPLookupResponse, error) {
	return specificResponse.transform()
}

/*
ipApiCo Represents ipapi.co

Rate limit: 30000 req/month (1000 req/day)

Test it:

curl -H "Accept: application/json" https://ipapi.co/json
*/
type ipApiCo struct{}

var IpApiCo = ipApiCo{}
var _ PublicIPProvider[genericPublicIPLookupResponse] = ipApiCo{}

func (p ipApiCo) URL() *url.URL                        { return mustParseURL("https://ipapi.co/json") }
func (p ipApiCo) AdditionalHeaders() map[string]string { return map[string]string{} }
func (p ipApiCo) Transform(specificResponse genericPublicIPLookupResponse) (*PublicIPLookupResponse, error) {
	return specificResponse.transform()
}

/*

Represents ip-api.com (TLS not supported in this free API)

Rate limit: 45 req/minute

Test it:

curl -H "Accept: application/json" http://ip-api.com/json?fields=query,as
*/

type (
	ipApiCom struct{}

	ipApiComResponse struct {
		ASN string `json:"as"`
		IP  string `json:"query"`
	}
)

// AdditionalHeaders implements [PublicIPProvider].
func (i ipApiCom) AdditionalHeaders() map[string]string {
	return map[string]string{}
}

// Transform implements [PublicIPProvider].
func (i ipApiCom) Transform(specificResponse ipApiComResponse) (*PublicIPLookupResponse, error) {
	// ASN from this provider looks like 'AS3320 Deutsche Telekom AG'
	// we only interseted in the first part
	parts := strings.Split(specificResponse.ASN, " ")
	if len(parts) < 1 {
		slog.Error("invlaid ASN", "asn", specificResponse.ASN)
		return nil, ErrInvalidASN
	}

	return ParsePublicIPLookupResponse(specificResponse.IP, parts[0])
}

// URL implements [PublicIPProvider].
func (i ipApiCom) URL() *url.URL {
	return mustParseURL("http://ip-api.com/json?fields=query,as")
}

var (
	_        PublicIPProvider[ipApiComResponse] = ipApiCom{}
	IpApiCom                                    = ipApiCom{}
)

/*

Represents ipwho.is

Rate Limit: 10_000 req/month

Test it:

curl -H "Accept: application/json" https://ipwho.is
*/

type (
	ipWhoIs struct{}

	ipWhoIsResponse struct {
		IP         string                    `json:"ip"`
		Connection ipWhoIsConnectionResponse `json:"connection"`
	}

	ipWhoIsConnectionResponse struct {
		ASN int `json:"asn"`
	}
)

// AdditionalHeaders implements [PublicIPProvider].
func (i ipWhoIs) AdditionalHeaders() map[string]string {
	return map[string]string{}
}

// Transform implements [PublicIPProvider].
func (i ipWhoIs) Transform(specificResponse ipWhoIsResponse) (*PublicIPLookupResponse, error) {
	return ParsePublicIPLookupResponse(specificResponse.IP, fmt.Sprintf("AS%d", specificResponse.Connection.ASN))
}

// URL implements [PublicIPProvider].
func (i ipWhoIs) URL() *url.URL {
	return mustParseURL("https://ipwho.is")
}

var (
	_       PublicIPProvider[ipWhoIsResponse] = ipWhoIs{}
	IpWhoIs                                   = ipWhoIs{}
)

/*

Represents my-ip.io

Rate limit: unknown

Test it:
curl -H "Accept: application/json" -H "User-Agent: vpn-killswitch/0.1.0" https://api.my-ip.io/v2/ip.json

*/

type (
	myIpIo struct{}

	myIpIoResponse struct {
		IP  string            `json:"ip"`
		ASN myIpIoASNResponse `json:"asn"`
	}

	myIpIoASNResponse struct {
		Number int `json:"number"`
	}
)

// AdditionalHeaders implements [PublicIPProvider].
func (m myIpIo) AdditionalHeaders() map[string]string {
	return map[string]string{}
}

// Transform implements [PublicIPProvider].
func (m myIpIo) Transform(specificResponse myIpIoResponse) (*PublicIPLookupResponse, error) {
	return ParsePublicIPLookupResponse(specificResponse.IP, fmt.Sprintf("AS%d", specificResponse.ASN.Number))
}

// URL implements [PublicIPProvider].
func (m myIpIo) URL() *url.URL {
	return mustParseURL("https://api.my-ip.io/v2/ip.json")
}

var (
	_      PublicIPProvider[myIpIoResponse] = myIpIo{}
	MyIpIo                                  = myIpIo{}
)
