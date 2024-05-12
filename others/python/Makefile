SOURCE_DIRS := moon tests

.PHONY: all
all: run

.PHONY: clean
clean: ## Clean project files
	@rm -rf ./var/htmlcov
	@rm -rf ./.coverage
	@rm -rf ./.ruff_cache/
	@rm -rf ./__pycache__/
	@find $(SOURCE_DIRS) -name "__pycache__" -prune -exec rm -rf {} \;
	@find $(SOURCE_DIRS) -name "*.py[co]" -prune -exec rm -rf {} \;
	@find $(SOURCE_DIRS) -name "*.so" -prune -exec rm -rf {} \;

.PHONY: r
r: run
.PHONY: run
run: ## Build and run program
	@python -m moon.moon

.PHONY: check
check: ## Most stringent checks (includes checks still in development)
	@pre-commit run ruff --all-files
	@pre-commit run ruff-format --all-files
	@pre-commit run docformatter --all-files
	@pre-commit run pyright --all-files
	@make test

.PHONY: t
t: test
.PHONY: test
test: ## Run unit tests
	@python -m unittest

.PHONY: c
c: coverage
.PHONY: coverage
coverage: ## Unit tests coverage report
	@rm -rf var/htmlcov
	@python -m coverage run -m unittest
	@python -m coverage html -d var/htmlcov
	@#python -m coverage report
	@open var/htmlcov/index.html || xdg-open var/htmlcov/index.html || :

.PHONY: coverage-pct
coverage-pct: ## Ensure code coverage of 100%
	@python -m coverage run -m unittest > /dev/null 2>&1 || :
	@python -m coverage json -q -o /dev/stdout | python -c \
			'import decimal, json, sys; \
			coverage = json.loads(input(), parse_float=decimal.Decimal); \
			percent_covered = coverage["totals"]["percent_covered"]; \
			print(percent_covered); \
			sys.exit(0 if percent_covered >= 99 else 1);'
