clean:
	rm -rf neon_daemon_data
seed:
	cargo test --test mgmt_api_test -- --nocapture