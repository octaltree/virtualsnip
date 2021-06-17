function! s:sep() abort
  if has('win32')
    return '\'
  else
    return '/'
  endif
endfunction

function! virtualsnip#path#core() abort
  let s = s:sep()
  let target = g:virtualsnip#root_dir . s . 'core' . s . 'target'
  let dir = target . s . 'release'
  if has('win32')
    return dir . s . 'virtualsnip.exe'
  else
    return dir . s . 'virtualsnip'
  endif
endfunction
