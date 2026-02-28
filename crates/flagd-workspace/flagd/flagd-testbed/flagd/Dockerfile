FROM ghcr.io/open-feature/flagd:v0.12.9 AS flagd

# Build stage
FROM golang:1.24 AS builder

WORKDIR /app

# Copy Go modules and dependencies
COPY launchpad/go.mod launchpad/go.sum ./
RUN go mod download

COPY launchpad/ ./

# Build the Go binary
RUN go build -o launchpad main.go


FROM alpine/openssl AS certs

# Copy the server and CA certificates
COPY ssl/* ./

RUN openssl genpkey -algorithm RSA -out server-key.pem -pkeyopt rsa_keygen_bits:2048
RUN openssl req -new -key server-key.pem -out server.csr -subj "/CN=localhost" \
   && openssl x509 -req -in server.csr -CA custom-root-cert.crt -CAkey custom-ca.key -CAcreateserial -out server-cert.pem -days 365 -sha256

FROM busybox:1.37 AS testbed

COPY --from=flagd /flagd-build /flagd
COPY --from=builder /app/launchpad /launchpad
COPY flags/* ./rawflags/
COPY launchpad/configs /configs

# Copy the custom root CA certificate into the image

COPY --from=certs server-cert.pem /ssl/
COPY --from=certs server-key.pem /ssl/
COPY --from=certs custom-root-cert.crt /ssl/

RUN mkdir "flags"


LABEL org.opencontainers.image.source="https://github.com/open-feature/flagd-testbed"

ENTRYPOINT ["./launchpad"]

