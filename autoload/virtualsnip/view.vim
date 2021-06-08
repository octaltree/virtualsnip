let s:virtualsnip_id = 1049
if exists('*nvim_create_namespace')
  let s:virtualsnip_id = nvim_create_namespace('virtualsnip')
endif

let g:virtualsnip#highlight_base = get(g:, 'virtualsnip#highlight_base', 'Comment')
let g:virtualsnip#sign = get(g:, 'virtualsnip#sign', ' ')

function! virtualsnip#view#get_current_buffer_info() abort
  let bufnr = bufnr('%')
  let lines_before_cursor = get(g:, 'virtualsnip#lines_before', 3)
  let cursor_line_no = line('.')
  let start_line_no = max([1, cursor_line_no - lines_before_cursor])
  let lines = getline(start_line_no, cursor_line_no)
  let start_line = start_line_no - 1
  let cursor_line = cursor_line_no - 1
  let sources = vsnip#source#find(bufnr('%'))
  " NOTE: start_line <= cursor_line < start_line + len(lines)
  return {
        \ 'highlight': {'base': g:virtualsnip#highlight_base},
        \ 'sign': g:virtualsnip#sign,
        \ 'lines': lines,
        \ 'start_line': start_line,
        \ 'cursor_line': cursor_line,
        \ 'sources': sources
        \}
endfunction

let s:shown = {}
" Refreshes virtualtexts if needed
function! virtualsnip#view#refresh(value) abort
  if type(a:value) != type({}) || !s:value_is_changed(a:value)
    return
  endif
  if s:value_is_blank(a:value)
    let s:shown = {}
    call nvim_buf_clear_namespace(0, s:virtualsnip_id, 0, -1)
    return
  endif
  " TODO: Is it faster to batch rewrite with c than to use vim script to diff and update?
  for action in s:diff(a:value)
    if action.op ==# 'delete' || action.op ==# 'update'
      call nvim_buf_clear_namespace(0, s:virtualsnip_id, action.line, action.line)
    endif
    if action.op ==# 'insert' || action.op ==# 'update'
      call nvim_buf_set_virtual_text(0, s:virtualsnip_id, action.line, action.chunks, {})
    endif
  endfor
endfunction

function! s:value_is_blank(value) abort
  return empty(a:value.texts)
endfunction

let s:last_value = {}
function! s:value_is_changed(value) abort
  if s:last_value == a:value
    return v:false
  else
    let s:last_value = a:value
    return v:true
  endif
endfunction

" return: action
" action: {
"   op: str,
"   line: index,
"   chunks
" }
function! s:diff(value) abort
  let res = []
  let this = s:value_to_dict(a:value)
  for l in keys(s:shown)
    if !has_key(this, l)
      call add(res, {'op': 'delete', 'line': str2nr(l), 'chunks': []})
    endif
  endfor
  for t in a:value.texts
    if !has_key(s:shown, t.line)
      call add(res, {'op': 'insert', 'line': t.line, 'chunks': t.chunks})
    elseif s:shown[t.line] != t.chunks
      call add(res, {'op': 'update', 'line': t.line, 'chunks': t.chunks})
    endif
  endfor
  let s:shown = this
  return res
endfunction

function! s:value_to_dict(value) abort
  let res = {}
  for t in a:value.texts
    let res[t.line] = t.chunks
  endfor
  return res
endfunction
