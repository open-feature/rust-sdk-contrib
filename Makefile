generate:
	git submodule update --init --recursive
	rm -f -r ./src/providers/flagd/proto
	cd ./src/providers/flagd/schemas && buf generate buf.build/open-feature/flagd --template protobuf/buf.gen.rust.yaml
