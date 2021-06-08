.PHONY: build
build:
	cd core && cargo build --release

.PHONY: clean
clean:
	rm -rf core/target/release
	rm -rf tools


# Development
.PHONY: dev
dev: vim rust

.PHONY: lint
lint: vim-lint rust-lint

.PHONY: d
d:
	watchexec 'make r lint

## rust {{{
.PHONY: rust
rust: rust-format rust-lint rust-test rust-doc

.PHONY: r
r: rust-lint rust-test rust-doc

.PHONY: rust-format
rust-format:
	cd core && cargo fmt

.PHONY: rust-lint
rust-lint:
	cd core && cargo clippy --all-targets

.PHONY: rust-test
rust-test:
	cd core && cargo test --all-targets

.PHONY: rust-doc
rust-doc:
	cd core && cargo doc
#}}}


## Vim {{{
.PHONY: vim
vim: vim-lint

.PHONY: vim-lint
vim-lint: tools/py/bin/vint
	./tools/py/bin/vint --version
	@./tools/py/bin/vint plugin
	@./tools/py/bin/vint autoload
# }}}


## Prepare tools {{{
prepare: tools/py/bin/vint tools/vim-themis

tools/vim-themis: tools
	git clone https://github.com/thinca/vim-themis $@

tools/py/bin/vint: tools/py/bin
	cd tools && ./py/bin/pip install vim-vint

tools/py/bin: tools
	cd tools && python -m venv py

tools:
	mkdir -p $@
# }}}

# vim: foldmethod=marker
