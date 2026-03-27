clean:
	rm -rf neon_daemon_data
seed:
	cargo test --test mgmt_api_test -- --nocapture
kill:
	kill $(lsof -i:50051 -t) || exit 0
	kill $(lsof -i:5430 -t) || exit 0
	kill $(lsof -i:5431 -t) || exit 0
	kill $(lsof -i:1234 -t) || exit 0
	kill $(lsof -i:1235 -t) || exit 0
	kill $(lsof -i:9993 -t) || exit 0