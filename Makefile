.PHONY: build
build:
	cd core && cargo build --release

.PHONY: clean
clean:
	rm -rf core/target/release
	rm -rf tools


# Development
.PHONY: dev
dev: vim lua rust

.PHONY: lint
lint: vim-lint lua-lint rust-lint

.PHONY: d
d:
	watchexec 'make r lint vim-test'

## rust {{{
.PHONY: rust
rust: rust-fmt rust-lint rust-test rust-doc

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
vim: vim-lint vim-test

.PHONY: vim-lint
vim-lint: tools/py/bin/vint
	./tools/py/bin/vint --version
	@./tools/py/bin/vint plugin
	@./tools/py/bin/vint autoload

.PHONY: vim-test
vim-test: tools/vim-themis
	THEMIS_VIM=nvim THEMIS_ARGS="-e --headless" tools/vim-themis/bin/themis tests/*.vim
# }}}


## Lua {{{
.PHONY: lua
lua: lua-format lua-lint

# https://github.com/Koihik/LuaFormatter
.PHONY: lua-format
lua-format:
	find lua tests -name "*.lua"| xargs lua-format -i

# https://github.com/mpeterv/luacheck
.PHONY: lua-lint
lua-lint:
	@find lua -name "*.lua"| xargs luacheck -q |\
		sed '/accessing undefined variable \[0m\[1mvim/d' |\
		sed '/unused argument \[0m\[1m_/d' |\
		sed '/^$$/d' |\
		sed 's/\[0m\[31m\[1m[0-9]\+ warnings\[0m//g'|\
		sed '/^Total:/d'
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
