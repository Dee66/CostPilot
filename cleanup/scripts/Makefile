.PHONY: release build test clean

# Build CostPilot binary
build:
	cargo build --release

# Run tests
test:
	cargo test

# Create release artifacts (package + sign + verify)
release:
	@chmod +x packaging/core_release/release.sh
	@bash packaging/core_release/release.sh

# Clean build artifacts
clean:
	rm -rf dist/
	rm -rf staging/
	rm -rf target/
	cargo clean

# Quick verification of existing artifacts
verify:
	@chmod +x packaging/core_release/verify_artifact.sh
	@for artifact in dist/*.zip dist/*.tar.gz; do \
		if [ -f "$$artifact" ]; then \
			bash packaging/core_release/verify_artifact.sh linux-x64 "$$artifact"; \
		fi \
	done
