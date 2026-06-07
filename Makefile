### Prerequisite:
# - cargo-llvm-cov: cargo install cargo-llvm-cov

## Using cargo-llvm-cov to generate lcov.info and html coverage reports covering functions, lines, and regions coverages.
# `branch` uses nightly because cargo-llvm-cov --branch is unstable.

OUT_DIR := target/coverage
OUT_FILE := $(OUT_DIR)/lcov.info

test:
	@echo "Running tests..."
	@cargo test --all-features
	@cargo test -q --doc --all-features

lcov:
	@echo "Generating lcov.info..."
	@mkdir -p $(OUT_DIR)
	@cargo llvm-cov test --all-features \
		--output-path $(OUT_FILE) \
		--lcov \
		--ignore-filename-regex \
			"_test\.rs$$|\
			tests/|\
			examples/"

html:
	@echo "Generating html coverage report..."
	@mkdir -p $(OUT_DIR)
	@cargo llvm-cov test --all-features \
		--output-dir $(OUT_DIR) \
		--open \
		--ignore-filename-regex \
			"_test\.rs$$|\
			tests/|\
			examples/"

branch:
	@echo "Generate coverage with branch coverage..."
	@mkdir -p $(OUT_DIR)
	@cargo +nightly llvm-cov test --all-features \
		--output-dir $(OUT_DIR) \
		--open \
		--branch \
		--ignore-filename-regex \
			"_test\.rs$$|\
			tests/|\
			examples/"

all:
	@echo "Running all checks..."
	@echo "Running cargo check---------------------------------------------"
	@cargo check --all-features
	@sleep 1
	@echo "Running formatting----------------------------------------------"
	@cargo fmt --all
	@sleep 1
	@echo "Running clippy--------------------------------------------------"
	@cargo clippy --all-features -- -D warnings
	@sleep 1
	@echo "Running doc"
	@cargo doc --all-features --no-deps
	@sleep 1
	@echo "Running tests---------------------------------------------------"
	@cargo test --all-features
	@cargo test -q --doc --all-features
