init:
	SKIP_DOCKER=true ./scripts/init_db.sh

watch:
	RUST_LOG=trace cargo watch -x clippy -x fmt -x run | bunyan

test:
	TEST_LOG=true cargo test | bunyan

clean:
	cargo +nightly udeps