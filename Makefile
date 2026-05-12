.DEFAULT_GOAL := build

JOBS ?= 1
BUILD_TYPE ?= release

ifeq ($(shell uname),Darwin)
OPENSSL_PREFIX := $(shell brew --prefix openssl@3 2>/dev/null || brew --prefix openssl 2>/dev/null)
OPENSSL_LDFLAGS := -L$(OPENSSL_PREFIX)/lib
OPENSSL_CPPFLAGS := -I$(OPENSSL_PREFIX)/include
endif

NEON_CONTRIB_DIRS := bloom btree_gin btree_gist citext cube dblink \
    dict_int dict_xsyn earthdistance file_fdw fuzzystrmatch hstore \
    intagg intarray isn lo ltree pg_freespacemap pg_logicalinspect \
    pg_stat_statements pg_surgery pg_walinspect pgcrypto pgrowlocks \
    pgstattuple postgres_fdw seg sslinfo spi tablefunc tcn \
    tsm_system_rows tsm_system_time unaccent

NEON_CONTRIB_EXTRAS := xml2 uuid-ossp

vanillapg:
	cd postgres && LDFLAGS="$(OPENSSL_LDFLAGS)" CPPFLAGS="$(OPENSSL_CPPFLAGS)" ./configure --prefix=$(CURDIR)/neon/pg_install/vanilla_v17 --without-icu --with-openssl
	$(MAKE) MAKELEVEL=0 -C postgres -j $(JOBS)
	$(MAKE) MAKELEVEL=0 -C postgres install

neon-contrib:
	@set -e; \
	for ver in v14 v15 v16 v17; do \
	  for ext in $(NEON_CONTRIB_DIRS); do \
	    dir="$(CURDIR)/neon/build/$$ver/contrib/$$ext"; \
	    if [ -d "$$dir" ]; then \
	      echo "==> installing $$ext for $$ver"; \
	      $(MAKE) MAKELEVEL=0 -C "$$dir" -j $(JOBS) install; \
	    fi; \
	  done; \
	done

neon-contrib-extras:
	@set -e; \
	for ver in v14 v15 v16 v17; do \
	  build="$(CURDIR)/neon/build/$$ver"; \
	  [ -d "$$build" ] || continue; \
	  echo "==> reconfiguring $$ver with --with-libxml --with-uuid=e2fs"; \
	  ( cd "$$build" && \
	    eval "../../vendor/postgres-$$ver/configure $$(./config.status --config) --with-libxml --with-uuid=e2fs" > reconfigure.log ); \
	  $(MAKE) MAKELEVEL=0 -C "$$build" clean; \
	  $(MAKE) MAKELEVEL=0 -C "$$build" -j $(JOBS) install; \
	  for ext in $(NEON_CONTRIB_EXTRAS); do \
	    dir="$$build/contrib/$$ext"; \
	    if [ -d "$$dir" ]; then \
	      echo "==> installing $$ext for $$ver"; \
	      $(MAKE) MAKELEVEL=0 -C "$$dir" -j $(JOBS) install; \
	    fi; \
	  done; \
	done

vector:
	@set -e; \
	if [ ! -f "$(CURDIR)/pgvector/Makefile" ]; then \
	  echo "pgvector submodule is missing; run 'git submodule update --init pgvector'" >&2; \
	  exit 1; \
	fi; \
	for ver in v14 v15 v16 v17; do \
	  pg_config="$(CURDIR)/neon/pg_install/$$ver/bin/pg_config"; \
	  [ -x "$$pg_config" ] || continue; \
	  echo "==> building pgvector for $$ver"; \
	  $(MAKE) MAKELEVEL=0 -C "$(CURDIR)/pgvector" USE_PGXS=1 PG_CONFIG=$$pg_config clean; \
	  $(MAKE) MAKELEVEL=0 -C "$(CURDIR)/pgvector" USE_PGXS=1 PG_CONFIG=$$pg_config -j $(JOBS) install; \
	done

build:
	cd web && yarn && yarn build
	$(MAKE) vanillapg
	$(MAKE) -C neon -j $(JOBS) -s CARGO_BUILD_JOBS=$(JOBS) BUILD_TYPE=$(BUILD_TYPE)
	$(MAKE) neon-contrib
	$(MAKE) neon-contrib-extras
	$(MAKE) vector
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
