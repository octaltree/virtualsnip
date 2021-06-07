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
      call add(res, vsnip#snippet#node#create_from_ast(ast))
    endfor
  endfor
  return res
endfunction
