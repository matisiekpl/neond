.DEFAULT_GOAL := build

JOBS ?= 1
BUILD_TYPE ?= release

build:
	$(MAKE) -C neon -j $(JOBS) -s CARGO_BUILD_JOBS=$(JOBS) BUILD_TYPE=$(BUILD_TYPE)
	CARGO_BUILD_JOBS=$(JOBS) RUSTFLAGS="-C link-arg=-Wl,--no-relax -C code-model=medium" cargo build --jobs $(JOBS) $(if $(filter release,$(BUILD_TYPE)),--release,)

kill:
	-kill $(shell lsof -ti:50051) 2>/dev/null || true
	-kill $(shell lsof -ti:5430)  2>/dev/null || true
	-kill $(shell lsof -ti:5431)  2>/dev/null || true
	-kill $(shell lsof -ti:1234)  2>/dev/null || true
	-kill $(shell lsof -ti:1235)  2>/dev/null || true
	-kill $(shell lsof -ti:9993)  2>/dev/null || true