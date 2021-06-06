let g:virtualsnip#events = get(g:, 'virtualsnip#events', ['CompleteDone'])

let s:is_enabled = v:false

let s:virtualsnip_id = 1049
if exists('*nvim_create_namespace')
  let s:virtualsnip_id = nvim_create_namespace('virtualsnip')
endif

function! virtualsnip#enable() abort
  augroup virtualsnip
    autocmd!
    autocmd InsertEnter * call s:on_event('InsertEnter')
    autocmd CursorMovedI * call s:on_event('CursorMovedI')
    autocmd InsertLeave * call s:clear()
  augroup END
  for event in g:virtualsnip#events
    if exists('##' . event)
      execute printf('autocmd virtualsnip %s * call s:on_event("%s")',
            \ event, event)
    endif
  endfor
  let s:is_enabled = v:true
endfunction

function! virtualsnip#disable() abort
  augroup virtualsnip
    autocmd!
  augroup END
  let s:is_enabled = v:false
endfunction

function! virtualsnip#is_enabled() abort
  return s:is_enabled
endfunction

function! s:clear() abort
  call nvim_buf_clear_namespace(bufnr('%'), s:virtualsnip_id, 0, -1)
endfunction

function! s:on_event(event) abort
  "let context = get_context()
  "if empty(context)
  "  return
  "endif
  "echomsg context
endfunction
