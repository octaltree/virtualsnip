let _ = exists('g:loaded_virtualsnip') && finish
let g:loaded_virtualsnip = 1

let g:virtualsnip#enable_at_startup = get(g:, 'echodoc#enable_at_startup', v:false)

if g:virtualsnip#enable_at_startup
  augroup virtualsnip
    autocmd!
    autocmd InsertEnter * call virtualsnip#enable()
  augroup END
endif
