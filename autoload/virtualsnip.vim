let g:virtualsnip#events = get(g:, 'virtualsnip#events', ['CompleteDone'])

let s:is_enabled = v:false

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
  let s:last_world = {}
  call virtualsnip#view#refresh({'texts': []})
endfunction

function! s:calc(world) abort
  let sh = virtualsnip#path#core()
  let json = system(sh, json_encode(a:world))
  return json_decode(json)
endfunction

function! s:on_event(event) abort
  let world = virtualsnip#view#get_current_buffer_info()
  if !world || !s:world_is_changed(world)
    return
  endif
  let value = s:calc(world)
  call virtualsnip#view#refresh(value)
endfunction

let s:last_world = {}
function! s:world_is_changed(world) abort
  if type(a:world) != type({})
    return v:false
  end
  if s:last_world == a:world
    return v:false
  else
    let s:last_world = a:world
    return v:true
  endif
endfunction
