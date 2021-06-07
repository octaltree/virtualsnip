local snippet = require('virtualsnip.snippet')
local function empty(table) return next(table) == nil end

local function copy(xs, from, to)
    -- lua 5.4 table.unpack
    return {unpack(xs, from, to)}
end

local function lines_to_string(ss) return join(ss) .. '\n' end

-- Returns new snippets state from view state includes current buffer and cursor pos
-- world: {
--   highlight,
--   lines: [str],
--   start_line: index,
--   cursor_line: index,
--   snippets
-- } # start_line <= cursor_line < start_line + len(lines)
-- highlight: {
--   base: str
-- }
-- index: int # zero-indexed
-- return: {
--   texts: [
--     text: {
--       line: index,
--       chunks: [[text:str, hl_group:str]]
--     }
--   ]
-- }
local function update(world)
    if empty(world.snippets) then return {texts = {}} end
    local num = world.cursor_line - world.start_line + 1
    local before_cursor_inclusive = copy(world.lines, 1, num)
    local matched = match(lines_to_string(before_cursor_inclusive), snippets)
    return {texts = {}}
end

local function match(buf, snippets)
    for snip in snippets do
        for node in snip do
        end end
end

-- {[("'

return {update = update}
