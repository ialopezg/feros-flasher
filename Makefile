CARGO ?= cargo
PYTHON ?= python3

BIN_ROOT ?= bin
GIT_COMMIT := $(shell git rev-parse HEAD 2>/dev/null || printf "none")
BUILD_TIMESTAMP := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || printf "unknown")

UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Darwin)
HOST_OS := darwin
EXECUTABLE_SUFFIX :=
else ifeq ($(UNAME_S),Linux)
HOST_OS := linux
EXECUTABLE_SUFFIX :=
else
HOST_OS := windows
EXECUTABLE_SUFFIX := .exe
endif

ifeq ($(UNAME_M),arm64)
HOST_ARCH := aarch64
else ifeq ($(UNAME_M),aarch64)
HOST_ARCH := aarch64
else ifeq ($(UNAME_M),x86_64)
HOST_ARCH := x86_64
else
HOST_ARCH := $(UNAME_M)
endif

BUILD_EXECUTABLE := target/release/flasher$(EXECUTABLE_SUFFIX)
PUBLISHED_DIR := $(BIN_ROOT)/$(HOST_OS)/$(HOST_ARCH)
PUBLISHED_EXECUTABLE := \
	$(PUBLISHED_DIR)/flasher$(EXECUTABLE_SUFFIX)

.PHONY: all check test fmt release verify package clean help

all: release

check:
	@$(CARGO) check \
		--manifest-path Cargo.toml \
		--all-targets --locked

test:
	@$(CARGO) test \
		--manifest-path Cargo.toml \
		--all-targets --locked

fmt:
	@$(CARGO) fmt \
		--manifest-path Cargo.toml \
		--all \
		--check

release:
	@printf "FeROS Flasher: building release for %s/%s...\n" \
		"$(HOST_OS)" \
		"$(HOST_ARCH)"
	@FEROS_BUILD_COMMIT="$(GIT_COMMIT)" \
		FEROS_BUILD_TIMESTAMP="$(BUILD_TIMESTAMP)" \
		$(CARGO) build \
		--manifest-path Cargo.toml \
		--release \
		--locked
	@mkdir -p $(PUBLISHED_DIR)
	@install -m 0755 \
		$(BUILD_EXECUTABLE) \
		$(PUBLISHED_EXECUTABLE)
	@printf "FeROS Flasher: generated %s\n" "$(PUBLISHED_EXECUTABLE)"

verify:
	@$(PYTHON) scripts/check.py

package:
	@$(PYTHON) scripts/package.py $(if $(TAG),--tag $(TAG),)

clean:
	@$(CARGO) clean \
		--manifest-path Cargo.toml
	@printf "FeROS Flasher: removed Cargo build artifacts.\n"

help:
	@printf "\nFeROS Flasher\n\n"
	@printf "  make check     Check all Rust targets\n"
	@printf "  make test      Run the test suite\n"
	@printf "  make fmt       Verify Rust formatting\n"
	@printf "  make release   Build and publish the host executable\n"
	@printf "  make verify    Run native validation and isolated CLI tests\n"
	@printf "  make package   Package a clean tested candidate (TAG=vX.Y.Z)\n"
	@printf "  make clean     Remove Cargo build artifacts\n\n"
