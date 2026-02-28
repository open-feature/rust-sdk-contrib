commands used to generate the cert

```shell
# generating custom ca
openssl genpkey -algorithm RSA -out custom-ca.key -pkeyopt rsa_keygen_bits:4096    

# generating root cert                                           
openssl req -x509 -new -key custom-ca.key -out custom-root-cert.crt -days 3650 -sha256 -subj "/CN=Flagd testbed ROOT CA"

```
