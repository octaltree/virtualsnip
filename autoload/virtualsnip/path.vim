function! s:sep() abort
  if has('win32')
    return '\'
  else
    return '/'
  endif
endfunction

function! virtualsnip#path#core() abort
  let s = s:sep()
  return g:virtualsnip#root_dir . s . 'core' . s . 'target' .
        \ s . 'release' . s . 'virtualsnip'
endfunction
