# schemas

Schemas and spec files pertaining primarily to [flagd](https://github.com/open-feature/flagd).
The Schema is available in the Buf Schema Registry, and can be found [here](https://buf.build/open-feature/flagd).

<br />
When the below commands are used the generated code is placed in the parent directory
<br />
<br />

## Requirements

- A Go installation
- You must have your `GOPATH` environment variable set in order to run these scripts.

The [Makefile](./Makefile) utilizes [Go](https://go.dev/) to install [Buf](https://buf.build/product/cli/) to the `GOPATH` and execute the binary directly.

### Setting up GOPATH

If you don't want to install `buf` to your global GOPATH, you can also set it for this repository only:

```
export GOPATH="$(pwd)/vendor/"
```

## Golang:

Generate grpc-gateway, go-grpc and go stubs

```
make gen-go-server
```

Generate go-grpc and go stubs

```
make gen-go
```

go package import

```
go get go.buf.build/grpc/go/open-feature/flagd
```

<br />

## Typescript:

Generate TypeScript stubs for grpc client and http/grpc input/output schema

```
make gen-ts
```

<br />

## Java:

generate Java stubs for grpc client and http/grpc input/output schema

```
make gen-java
```

## CSharp

generate csharp stubs for grpc client and http/grpc input/output schema
```
make gen-csharp
```

## PHP:

Generate PHP stubs for grpc client

```
make gen-php
```


## Ruby:

Generate Ruby stubs for grpc client

```
make gen-ruby
```


## Rust:

Generate Rust stubs for grpc client

```
make gen-rust
```
