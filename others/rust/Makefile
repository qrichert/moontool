PREFIX ?= /usr/local

.PHONY: all
all: build

.PHONY: clean
clean: ## Clean project files
	@cargo clean

.PHONY: r
r: run
.PHONY: run
run: ## Build and run program
	@cargo run --quiet --features="rich-output"

.PHONY: b
b: build
.PHONY: build
build: ## Make optimized release build
	@cargo build --release --features="rich-output"

.PHONY: check
check: ## Most stringent checks (includes checks still in development)
	@rustup update
	@cargo fmt
	@cargo doc --no-deps --all-features
	@cargo check
	@cargo clippy --all-targets --all-features -- -D warnings -W clippy::all -W clippy::cargo -W clippy::complexity -W clippy::correctness -W clippy::nursery -W clippy::pedantic -W clippy::perf -W clippy::style -W clippy::suspicious -A clippy::missing-const-for-fn -A clippy::option-if-let-else -A clippy::suboptimal_flops
	@make test

.PHONY: t
t: test
.PHONY: test
test: ## Run unit tests
	@cargo test --features="rich-output"

.PHONY: doc
doc: ## Build documentation
	@cargo doc

.PHONY: c
c: coverage
.PHONY: coverage
coverage: ## Unit tests coverage report
	@cargo tarpaulin --engine Llvm --timeout 120 --out Html --output-dir target/ --features="rich-output"
	@open target/tarpaulin-report.html || xdg-open target/tarpaulin-report.html || :

.PHONY: coverage-pct
coverage-pct: ## Ensure code coverage of 100%
	@coverage=$$(cargo tarpaulin --engine Llvm --out Stdout --features="rich-output" 2>&1); \
		percent_covered=$$(echo "$$coverage" | grep -o '^[0-9]\+\.[0-9]\+% coverage' | cut -d'%' -f1); \
		echo $$percent_covered; \
		[ $$(echo "$$percent_covered == 100" | bc -l) -eq 0 ] && exit 1; \
		exit 0

.PHONY: install
install: ## Install moontool
	install -d $(PREFIX)/bin/
	install ./target/release/moontool $(PREFIX)/bin/moontool
