function! s:node(n) abort
  if a:n.type ==# 'text'
    return {'type':'text', 'value':a:n.value}
  elseif a:n.type ==# 'placeholder'
    let children = []
    for c in a:n.children
      call add(children, s:node(c))
    endfor
    return {'type':'placeholder', 'children':children}
  elseif a:n.type ==# 'variable'
    let children = []
    for c in a:n.children
      call add(children, s:node(c))
    endfor
    return {'type':'variable', 'children':children}
  endif
endfunction

" sources: [{body: [str]}]
" return: snippets
" snippets: [
"   snippet: [
"     node: placeholder|variable|text
"   ]
" ]
" placeholder: {
"   uid: int,
"   type: 'placeholder',
"   id: int,
"   is_final: bool,
"   follower: bool,
"   choice: [?],
"   children: [node]
" }
" variable: {
"   uid: int,
"   type: 'variable',
"   name: str,
"   unknown: bool,
"   resolver: null|{func: f, once: bool}
"   children: [node]
" }
" text: {
"   uid: int,
"   type: 'text',
"   value: str,
"   children: []
"  }
function! virtualsnip#model#snippets_from_sources(sources) abort
  let res = []
  for snippets in a:sources
    for snippet in snippets
      let s = join(snippet.body, '\n')
      let ast = vsnip#snippet#parser#parse(s)
      let nodes = []
      for n in vsnip#snippet#node#create_from_ast(ast)
        call add(nodes, s:node(n))
      endfor
      call add(res, nodes)
    endfor
  endfor
  return res
endfunction
