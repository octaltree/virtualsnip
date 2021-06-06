" Returns new snippets state from view state includes current buffer and cursor pos
" world: {
"   lines: [str],
"   start_line: index,
"   cursor_line: index,
"   sources: [
"     source: [
"       snippets: [
"         snippet
"       ]
"     ]
"   ]
" } # start_line <= cursor_line < start_line + len(lines)
" index: int # zero-indexed
" snippet: {
"   body: [str],
"   description: str,
"   lael: str,
"   prefix: [str],
"   prefix_alias: [str],
" }
" return: {
"   texts: [
"     text: {
"       line: index,
"       chunks: [[text:str, hl_group:str]]
"     }
"   ]
" }
function! virtualsnip#model#update(world) abort
  echomsg a:world
endfunction
