# Makefile — ForgottenServer Rust port
#
# Top-level targets for developers and the harness. Real builds use
# cargo / docker compose directly; these targets are convenience wrappers.

.PHONY: help harness harness-up harness-down test clippy fmt ledger ledger-test ledger-build ledger-rollup ledger-cross e2e flow flow-test flow-build flow-curate flow-check-network flow-curate-events flow-check-events

help:
	@echo "Available targets:"
	@echo "  make harness        — run the equivalence harness (all lanes)"
	@echo "  make harness-up     — bring up the side-by-side docker stack only"
	@echo "  make harness-down   — tear down the side-by-side docker stack"
	@echo "  make test           — cargo test --lib --workspace"
	@echo "  make clippy         — cargo clippy --workspace --lib --tests -- -D warnings"
	@echo "  make fmt            — cargo fmt --all"
	@echo "  make ledger         — validate MIGRATION_LEDGER.yml + files: rollup"
	@echo "  make ledger-test    — run scripts/ledger/ unit tests"
	@echo "  make ledger-build   — regenerate MIGRATION_LEDGER.yml from manifests"
	@echo "  make ledger-rollup  — regenerate files: rollup from symbol rows"
	@echo "  make ledger-cross   — phase-2 cross-validation (roundtrip + coverage + orphans)"
	@echo "  make flow           — validate flow graph (node keys, dangling edges, orphans)"
	@echo "  make flow-build     — bootstrap nodes + extract static edges from C++ source"
	@echo "  make flow-curate    — apply curated network edges (opcode dispatch table)"
	@echo "  make flow-check-network — verify every active opcode has a curated edge"
	@echo "  make flow-curate-events — apply curated event/scheduler edges"
	@echo "  make flow-check-events  — verify event type and scheduler tick coverage"
	@echo "  make flow-test      — run scripts/flow/ unit tests"
	@echo ""
	@echo "Harness lane subset:"
	@echo "  HARNESS_LANES=wire_replay,otbm_diff make harness"

harness:
	@bash scripts/harness/run.sh

harness-up:
	@bash -c 'source scripts/harness/lib.sh && harness::up && harness::ready'

harness-down:
	@bash -c 'source scripts/harness/lib.sh && harness::down'

test:
	cargo test --lib --workspace

e2e:
	cargo test -p forgottenserver-e2e --features e2e -- --test-threads=1

clippy:
	cargo clippy --workspace --lib --tests -- -D warnings

fmt:
	cargo fmt --all

ledger:
	@python3 -m unittest discover -s scripts/ledger/tests >/dev/null
	@python3 scripts/ledger/validate.py
	@python3 scripts/ledger/rollup.py --check

ledger-test:
	@python3 -m unittest discover -s scripts/ledger/tests -v

ledger-build:
	@python3 scripts/ledger/build_seed.py

ledger-rollup:
	@python3 scripts/ledger/rollup.py

ledger-cross:
	@python3 scripts/ledger/cross_validate.py

flow:
	@python3 scripts/flow/validate.py
	@python3 scripts/flow/check_network_coverage.py
	@python3 scripts/flow/check_event_coverage.py

flow-build:
	@python3 scripts/flow/bootstrap_nodes.py
	@python3 scripts/flow/build_edges.py

flow-curate:
	@python3 scripts/flow/curate_network.py

flow-check-network:
	@python3 scripts/flow/check_network_coverage.py

flow-curate-events:
	@python3 scripts/flow/curate_events.py

flow-check-events:
	@python3 scripts/flow/check_event_coverage.py

flow-test:
	@python3 -m unittest discover -s scripts/flow/tests -v
