let s:virtualsnip_id = 1049
if exists('*nvim_create_namespace')
  let s:virtualsnip_id = nvim_create_namespace('virtualsnip')
endif

function! virtualsnip#view#get_current_buffer_info() abort
  let bufnr = bufnr('%')
  let max_lines = get(g:, 'virtualsnip#max_lines', 5)
  let cursor_line_no = line('.')
  let start_line_no = max([1, cursor_line_no - max_lines + 1])
  let lines = getline(start_line_no, cursor_line_no)
  let start_line = start_line_no - 1
  let cursor_line = cursor_line_no - 1
  let sources = vsnip#source#find(bufnr('%'))
  " start_line <= cursor_line < start_line + len(lines)
  return {
        \ 'lines': lines,
        \ 'start_line': start_line,
        \ 'cursor_line': cursor_line,
        \ 'sources': sources
        \}
endfunction

" Refreshes virtualtexts if needed
function! virtualsnip#view#refresh(value) abort
  if !s:value_is_changed(a:value)
    return
  endif
  if s:value_is_blank(a:value)
    call nvim_buf_clear_namespace(0, s:virtualsnip_id, 0, -1)
    return
  endif
endfunction

function! s:value_is_blank(value) abort
  return empty(a:value.texts)
endfunction

let s:last_value = {}
function! s:value_is_changed(value) abort
  if type(a:value) != type({})
    return v:false
  end
  if s:last_value == a:value
    return v:true
  else
    let s:last_value = a:value
    return v:false
  endif
endfunction
