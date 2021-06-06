# virtualsnip
This plugin shows snippets as virtualtext on neovim.

## Requirements
* [vim-vsnip](https://github.com/hrsh7th/vim-vsnip)
  - Use it as a library so you don't have to bind keys for snippets
* Some snippet sources for vim-vsnip

## Installation
For dein
```
call dein#add('hrsh7th/vim-vsnip')
call dein#add('octaltree/virtualsnip')
```

## Config
```
let g:virtualsnip#enable_at_startup = 1
```
