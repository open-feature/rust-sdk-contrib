BUF_VERSION=v1.26.1

guard-%:
	@ if [ "${${*}}" = "" ]; then \
        echo "Environment variable $* not set"; \
        exit 1; \
    fi

install-buf:
	go install github.com/bufbuild/buf/cmd/buf@${BUF_VERSION}

install-yq:
	go install github.com/mikefarah/yq/v4@latest

lint: guard-GOPATH
	cd protobuf && ${GOPATH}/bin/buf lint

gen-go: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.go.yaml

gen-go-server: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.go-server.yaml

gen-ts: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.ts.yaml

gen-java: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.java.yaml

gen-python: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.python.yaml

gen-csharp: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.csharp.yaml

gen-php: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.php.yaml

gen-ruby: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.ruby.yaml
	
gen-rust: install-buf guard-GOPATH
	${GOPATH}/bin/buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.rust.yaml

gen-schema-json: install-yq
	yq eval -o=json json/targeting.yaml > json/targeting.json
	yq eval -o=json json/flags.yaml > json/flags.json
	yq eval -o=json json/flagd.yaml > json/flagd.json
	
.PHONY: test
test: gen-schema-json ajv-validate-flagd-schema
	go test -v ./json

ajv-validate-flagd-schema:
	@if ! npm ls ajv-cli; then npm ci; fi
	npx ajv compile -s json/targeting.json
# load dependent schemas with -r
	npx ajv compile -r json/targeting.json -s json/flags.json
	npx ajv compile -r json/targeting.json -r json/flags.json -s json/flagd.json
