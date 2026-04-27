package main

import (
	"flag"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"
)

const minPollInterval = 5

var httpPort int
var asn string
var logLevel string
var maxAttempts int

func init() {
	flag.IntVar(&httpPort, "port", 8080, "http port for the server")
	flag.StringVar(&asn, "asn", "", "ASN of the VPN porvider")
	flag.StringVar(&logLevel, "log-level", "info", "log level")
	flag.IntVar(&maxAttempts, "max-attempts", 5, "how often should it be tried to reach a public ip provider")
}

func main() {
	flag.Parse()
	// take asn from the environment
	if asn == "" {
		if asnFromEnv, ok := os.LookupEnv("VPN_PROVIDER_ASN"); ok {
			asn = asnFromEnv
		} else {
			flag.Usage()
			os.Exit(1)
		}
	}

	defaultLogger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: stringToLogLevel(logLevel)}))
	slog.SetDefault(defaultLogger)

	http.HandleFunc("GET /health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("OK"))
	})

	asnChan := make(chan string)
	errChan := make(chan error)
	killswitchServed := make(chan struct{})

	http.HandleFunc("GET /killswitch", func(w http.ResponseWriter, r *http.Request) {
		select {
		case currentASN := <-asnChan:
			if currentASN == asn {
				w.WriteHeader(http.StatusOK)
			} else {
				w.WriteHeader(http.StatusServiceUnavailable)
			}
		case err := <-errChan:
			slog.Error("failed to get th ASN for the killswitch handler", "err", err)
			w.WriteHeader(http.StatusServiceUnavailable)
		}
		killswitchServed <- struct{}{}
	})

	var wg sync.WaitGroup
	wg.Add(2)

	// http server
	go func() {
		if err := http.ListenAndServe(fmt.Sprintf(":%d", httpPort), nil); err != nil {
			slog.Error("failed to start http server", "err", err)
		}
		wg.Done()
	}()

	// public ip lookup
	go func() {
		for {
			resp, err := LookupWithRandomProvider(maxAttempts)
			lastUpdated := time.Now()
			if err != nil {
				errChan <- err
				continue
			}

			asnChan <- resp.ASN
			<-killswitchServed
			lastServed := time.Now()

			// adapts itslef to the polling interval
			// supposing the interval is being constant which it is if calling as k8s readiness/health endpoint
			// tries to get new data 1 second before it's going to be called
			pollInterval := int64(lastServed.Sub(lastUpdated).Seconds()) - 1
			if pollInterval < minPollInterval {
				pollInterval = minPollInterval
			}
			slog.Info("sleeping before fetching public ip...", "duration", pollInterval)
			time.Sleep(time.Duration(pollInterval) * time.Second)
		}
	}()

	slog.Info("started http server", "port", httpPort)
	wg.Wait()
}

func stringToLogLevel(s string) slog.Level {
	switch strings.ToUpper(s) {
	case "DEBUG":
		return slog.LevelDebug
	case "ERROR":
		return slog.LevelError
	case "WARN":
		return slog.LevelWarn
	default:
		return slog.LevelInfo
	}
}
