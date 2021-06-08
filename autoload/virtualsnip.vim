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
  echomsg "clear"
  " TODO: if the display is delayed, it will appear after clear
  call virtualsnip#view#refresh({'texts': []})
endfunction

let s:connection = 0
function! s:start() abort
  if s:connection
    return s:connection
  end
  let s:connection = jobstart([virtualsnip#path#core()], {'rpc': v:true})
  return s:connection
endfunction

function! s:calc(world) abort
  echomsg "write"
  let sh = virtualsnip#path#core() . ' ' . shellescape(json_encode(a:world))
  let json = system(sh)
  return json_decode(json)
endfunction

function! s:on_event(event) abort
  let start = reltime()
  let world = virtualsnip#view#get_current_buffer_info()
  if !s:world_is_changed(world)
    return
  endif
  "" FIXME: Too heavy 200~600ms
  let world['snippets'] = virtualsnip#model#snippets_from_sources(world.sources)
  call remove(world, 'sources')
  let value = s:calc(world)
  call virtualsnip#view#refresh(value)
  echomsg reltimestr(reltime(start))
endfunction

let s:last_world = {}
function! s:world_is_changed(world) abort
  if type(a:world) != type({})
    return v:false
  end
  if s:last_world == a:world
    return v:true
  else
    let s:last_world = a:world
    return v:false
  endif
endfunction
