" set verbose=1

let s:suite = themis#suite('compatibility')
let s:assert = themis#helper('assert')

function! s:suite.update() abort
  lua << EOF
    local virtualsnip = require('virtualsnip')
    local function f(s) return s end
    local world = {
        highlight = {base = 'Comment'},
        start_line = 2,
        cursor_line = 3,
        snippets = {{
            {
                children = {},
                uid = 3329,
                to_string = f('137'),
                type = 'text',
                text = f('136'),
                new = f('135'),
                value = 'if '
            }, {
                id = 1,
                follower = false,
                is_final = 0,
                uid = 1264,
                to_string = f('130'),
                type = 'placeholder',
                choice = {},
                text = f('129'),
                new = f('128'),
                children = {
                    {
                        children = {},
                        uid = 3330,
                        to_string = f('137'),
                        type = 'text',
                        text = f('136'),
                        new = f('135'),
                        value = 'condition'
                    }
                }
            }, {
                children = {},
                uid = 3331,
                to_string = f('137'),
                type = 'text',
                text = f('136'),
                new = f('135'),
                value = ' {\n    '
            }, {
                id = 2,
                follower = false,
                is_final = 0,
                uid = 1265,
                to_string = f('130'),
                type = 'placeholder',
                choice = {},
                text = f('129'),
                new = f('128'),
                children = {
                    {
                        children = {},
                        uid = 3332,
                        to_string = f('137'),
                        type = 'text',
                        text = f('136'),
                        new = f('135'),
                        value = 'unimplemented!();'
                    }
                }
            }, {
                children = {},
                uid = 3333,
                to_string = f('137'),
                type = 'text',
                text = f('136'),
                new = f('135'),
                value = '\n}'
            }
          }},
        lines = {'fn main(){', '    if'}
    }
    local function copy(xs, from, to)
        -- lua 5.4 table.unpack
        return {unpack(xs, from, to)}
    end
    num = world.cursor_line - world.start_line + 1
    local before_cursor_inclusive = copy(world.lines, 1, num)
    found = virtualsnip.find('if', world.snippets[1])
    contained = virtualsnip.contains('    if ', 'if')
    local matched = virtualsnip.match(before_cursor_inclusive, world.snippets)
    y = virtualsnip.update(world)
EOF
  call s:assert.equals(luaeval('num'), 2)
  call s:assert.equals(luaeval('found'), {'num': 3, 'hit': 1, 'num_first': 1})
  call s:assert.equals(luaeval('contained'), 4)
  call s:assert.equals(luaeval('y'), {
        \ 'texts': [
        \   {'line': 2, 'chunks': [['', 'Comment']]},
        \   {'line': 3, 'chunks': [["condition {\n    unimplemented!();\n}", 'Comment']]}]
        \})


endfunction
