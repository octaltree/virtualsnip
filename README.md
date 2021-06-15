# virtualsnip
This plugin shows snippets as virtualtext on neovim.

![gif](https://user-images.githubusercontent.com/7942952/122072863-eaecb480-ce32-11eb-9ad5-3f2295b477be.gif)

## Requirements
* Neovim
* [vim-vsnip](https://github.com/hrsh7th/vim-vsnip)
  - Use it as a library so you don't have to bind keys for snippets
* Some snippet sources for vim-vsnip

## Installation
For dein.toml
```toml
[[plugins]]
repo = 'hrsh7th/vim-vsnip'
[[plugins]]
repo = 'octaltree/virtualsnip'
build = 'make'
on_event = 'InsertEnter' # if lazy
hook_add='''
let g:virtualsnip#enable_at_startup = v:true
let g:virtualsnip#sign = ' » '
let g:virtualsnip#highlight_base = 'Comment'
'''
```
For other plugin managers, please do the `make` yourself.
