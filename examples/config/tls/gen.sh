#!/bin/bash
set -e

cd "$(dirname "$0")"
echo "Generating CA..."
openssl genrsa -out ca.key 2048
openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 -out ca.crt -subj "/CN=Switchboard CA"

echo "Generating Single Certificate (single.test.local)..."
openssl genrsa -out single.key 2048
openssl req -new -key single.key -out single.csr -subj "/CN=single.test.local"
openssl x509 -req -in single.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out single.crt -days 365 -sha256

echo "Generating SNI Certificate A (sni-a.test.local)..."
openssl genrsa -out sni-a.key 2048
openssl req -new -key sni-a.key -out sni-a.csr -subj "/CN=sni-a.test.local"
openssl x509 -req -in sni-a.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out sni-a.crt -days 365 -sha256

echo "Generating SNI Certificate B (sni-b.test.local)..."
openssl genrsa -out sni-b.key 2048
openssl req -new -key sni-b.key -out sni-b.csr -subj "/CN=sni-b.test.local"
openssl x509 -req -in sni-b.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out sni-b.crt -days 365 -sha256

echo "Cleaning up CSRs..."
rm *.csr

echo "Done."
