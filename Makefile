BENCH_DIR := core/qrd-core

.PHONY: benchmark benchmark-release benchmark-json benchmark-streaming

benchmark:
	cd $(BENCH_DIR) && cargo bench --workspace

benchmark-release:
	cd $(BENCH_DIR) && cargo build --workspace --release && cargo bench --workspace

benchmark-json:
	cd $(BENCH_DIR) && cargo bench --bench json_to_qrd_benchmark

benchmark-streaming:
	cd $(BENCH_DIR) && cargo bench --bench streaming_benchmark