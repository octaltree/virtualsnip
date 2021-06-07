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
        cursor_line = 2,
        snippets = {
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
        },
        lines = {'fn main(){', 'if'}
    }
    y = virtualsnip.update(world)
EOF
  call s:assert.equals(luaeval('y'), {
        \ 'texts': [{'line': 2, 'chunks': [['if {}', 'Comment']]}]
        \})
endfunction
