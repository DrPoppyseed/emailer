init:
	SKIP_DOCKER=true ./scripts/init_db.sh

watch:
	RUST_LOG=trace cargo watch -x clippy -x fmt -x run | bunyan

test:
	TEST_LOG=true cargo test | bunyan

clean:
	cargo +nightly udeps

# Build the `emailer` container
build:
	docker build --tag emailer .

# Build container with verbose option (shows command output)
build-v:
	DOCKER_BUILDKIT=0 docker build --tag emailer .

# Run containerized `emailer` rust app  
run: 
	docker run -p 8000:8000 emailer | bunyan