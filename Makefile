init:
	SKIP_DOCKER=true ./scripts/init_db.sh

watch:
	RUST_LOG=trace cargo watch -x run