.PHONY: clean
clean:
	rm -rf tools
	rm -rf core/targets


# Development
.PHONY: dev
dev: vim

.PHONY: d
d:
	watchexec 'make lint'


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
prepare: tools/py/bin/vint

tools/py/bin/vint: tools/py/bin
	cd tools && ./py/bin/pip install vim-vint

tools/py/bin: tools
	cd tools && python -m venv py

tools:
	mkdir -p $@
# }}}

# vim: foldmethod=marker
