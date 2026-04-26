package main

import (
	"reflect"
	"testing"
)

func TestParsePublicIPLookupResponse(t *testing.T) {
	tests := []struct {
		name    string
		ip      string
		asn     string
		want    *PublicIPLookupResponse
		wantErr error
	}{
		{
			name:    "invalid ip address",
			ip:      "bla",
			asn:     "AS3320",
			want:    nil,
			wantErr: ErrInvalidIP,
		},
		{
			name:    "empty asn",
			ip:      "127.0.0.1",
			asn:     "",
			want:    nil,
			wantErr: ErrInvalidASN,
		},
		{
			name:    "wrong asn prefix",
			ip:      "127.0.0.1",
			asn:     "sa3220",
			want:    nil,
			wantErr: ErrInvalidASN,
		},
		{
			name:    "short asn",
			ip:      "127.0.0.1",
			asn:     "as",
			want:    nil,
			wantErr: ErrInvalidASN,
		},
		{
			name:    "valid ip and asn",
			ip:      "127.0.0.1",
			asn:     "as3220",
			want:    &PublicIPLookupResponse{IP: []byte{127, 0, 0, 1}, ASN: "AS3220"},
			wantErr: nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ParsePublicIPLookupResponse(tt.ip, tt.asn)
			if !reflect.DeepEqual(tt.want, got) {
				t.Errorf("want '%v', got '%v'", tt.want, got)
			}

			if tt.wantErr != err {
				t.Errorf("want '%v' got '%v'", tt.wantErr, err)
			}
		})
	}
}
