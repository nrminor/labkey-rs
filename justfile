# labkey-rs project justfile
# All repeating commands should be recipes here.
# Agents MUST read and use these recipes.

# Default recipe: show available commands
default:
    @just --list

# choose recipes interactively
choose:
    @just --choose

# === Development Workflow ===

# Run all pre-commit checks (required before committing)
check: fmt-check lint test doc-check
    @echo "All checks passed"

# Run checks on all files (required before pushing)
check-all: fmt-check lint-all test-all doc-check
    @echo "All checks passed on full codebase"

# === Formatting ===

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Apply formatting fixes
fmt:
    cargo fmt --all

# === Linting ===

# Run clippy with deny warnings
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy on all files
lint-all:
    cargo clippy --all-targets --all-features -- -D warnings

# === Testing ===

# Run tests with nextest (--no-tests=pass allows empty test suites)
test:
    cargo nextest run --all-features --no-tests=pass

# Run all tests including ignored
test-all:
    cargo nextest run --all-features --run-ignored all --no-tests=pass

# Run tests with verbose output
test-verbose:
    cargo nextest run --all-features --no-capture --no-tests=pass

# === Building ===

# Build debug
build:
    cargo build

# Build release
build-release:
    cargo build --release

# Check compilation without building
check-compile:
    cargo check --all-targets --all-features

# === jj Workflow ===
# Since jj bypasses git hooks, use these recipes for enforcement.

# Prepare a commit: run all checks, then show status
prepare-commit: check
    @echo ""
    @echo "Ready to commit. Run: jj commit -m 'your message'"
    @jj status

# Prepare for push: run full checks
prepare-push: check-all
    @echo ""
    @echo "Ready to push. Run: jj git push"

# Show current jj status
status:
    jj status

# Show jj log
log:
    jj log

# === Utility ===

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# === Documentation ===

# Check that documentation builds without errors
doc-check:
    cargo doc --no-deps --document-private-items

# Generate and open documentation
doc:
    cargo doc --no-deps --open

# Count source lines of code (excluding blanks and comments)
sloc:
    @tokei --types=Rust --compact

# === Project Setup ===

# Full project setup for new clones (run this first!)
setup: clone-refs
    @echo ""
    @echo "Project setup complete!"
    @echo "Reference repo: .agents/repos/labkey-api-js/"

# === Reference Repositories ===

# Clone the upstream JS client for reference
clone-refs:
    @echo "Cloning reference repository into .agents/repos/..."
    @mkdir -p .agents/repos
    @echo "Cloning labkey-api-js (upstream JS/TS client)..."
    git clone https://github.com/LabKey/labkey-api-js.git .agents/repos/labkey-api-js || echo "labkey-api-js already exists, skipping"
    @echo "Reference repository cloned to .agents/repos/"

# Update reference repository to latest
update-refs:
    @echo "Updating reference repository..."
    cd .agents/repos/labkey-api-js && git pull || true
    @echo "Reference repository updated"

# Remove reference repository
clean-refs:
    @echo "Removing reference repository..."
    rm -rf .agents/repos
    @echo "Reference repository removed"
