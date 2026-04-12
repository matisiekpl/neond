.DEFAULT_GOAL := build

JOBS ?= 1
BUILD_TYPE ?= release

vanillapg:
	cd postgres && ./configure --prefix=$(CURDIR)/neon/pg_install/vanilla_v17 --without-icu
	$(MAKE) MAKELEVEL=0 -C postgres -j $(JOBS)
	$(MAKE) MAKELEVEL=0 -C postgres install

build:
	cd web && yarn && yarn build
	$(MAKE) vanillapg
	$(MAKE) -C neon -j $(JOBS) -s CARGO_BUILD_JOBS=$(JOBS) BUILD_TYPE=$(BUILD_TYPE)
	CARGO_BUILD_JOBS=$(JOBS) cargo build --jobs $(JOBS) $(if $(filter release,$(BUILD_TYPE)),--release,)

kill:
	-kill $(shell lsof -ti:50051) 2>/dev/null || true
	-kill $(shell lsof -ti:5430)  2>/dev/null || true
	-kill $(shell lsof -ti:5431)  2>/dev/null || true
	-kill $(shell lsof -ti:1234)  2>/dev/null || true
	-kill $(shell lsof -ti:1235)  2>/dev/null || true
	-kill $(shell lsof -ti:9993)  2>/dev/null || true

clean:
	make -C neon distclean
	make -C postgres clean
	rm -rf $(VANILLA_PG_BUILD) $(VANILLA_PG_INSTALL)
	cargo clean