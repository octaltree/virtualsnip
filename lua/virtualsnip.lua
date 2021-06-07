local function empty(table) return next(table) == nil end

local function copy(xs, from, to)
    -- lua 5.4 table.unpack
    return {unpack(xs, from, to)}
end

-- local function lines_to_string(ss) return join(ss) .. '\n' end

local function first_text(nodes)
    local res = {}
    local found = false
    for _, n in ipairs(nodes) do
        if found then
            table.insert(res, n)
        elseif n.type == 'text' then
            found = true
            table.insert(res, n)
        end
    end
    return res
end

local function split(s, sep)
    local res = {}
    for str in string.gmatch(s, "([^" .. sep .. "]+)") do
        table.insert(res, str)
    end
    return res
end

-- return: nil|index
local function contains(sentence, words)
    -- TODO: fool
    for i = 1, #sentence - #words + 1 do
        if string.sub(sentence, i, #words) == words then return i - 1 end
    end
    return nil
end

-- return: {num: int, hit: int, num_first}
local function find(line, nodes)
    if empty(nodes) or line == '' then
        return {hit = 0, num = 0, num_first = 0}
    end
    -- first node is text type
    local fs = split(nodes[1].value, '%s')
    local rest = copy(nodes, 2, -1)
    local cur = 1
    local hit = 0
    local num = 0
    for _, word in ipairs(fs) do
        num = num + 1
        local r = contains(string.sub(line, cur, -1), word)
        if r == nil then return {hit = hit, num = num, num_first = #fs} end
        hit = hit + 1
        cur = r + #word
    end
    for _, n in ipairs(rest) do
        if n.type == 'text' then
            num = num + 1
            local word = n.value
            local r = contains(string.sub(line, cur, -1), word)
            if r == nil then
                return {hit = hit, num = num, num_first = #fs}
            end
            hit = hit + 1
            cur = r + #word
        end
    end
    return {hit = hit, num = num, num_first = #fs}
end

-- Let buf[0][0] be level 0. Set the opening parentheses {[("' to +1.
-- Set inner parentheses ( to ) to the +1. Odd numbers in parentheses, even
-- numbers in scopes.
-- return: [{level: int, range: range}]
-- range: {line: index, col: index}
-- index: int # 0-indexed
-- local function calc_nest_level(buf)
--  if empty(buf) or #buf[0] == 0 then
--    return []
--  end
--  local level = 0
--  local start = {line = 0, col = 0}
--  -- FIXME: multibyte chars
--  for l in buf do
--    l:gsub('.', function(byte)
--    end)
--  end
-- end
-- return: [[node]]
local function match(buf, snippets)
    local snips
    do
        snips = {}
        for _, snip in ipairs(snippets) do
            -- NOTE: Can't use placeholder before keyword to find matching
            table.insert(snips, first_text(snip))
        end
    end
    local res = {}
    for _, l in ipairs(buf) do
        local founds
        do
            founds = {}
            for _, nodes in ipairs(snips) do
                table.insert(founds, find(l, nodes))
            end
        end
        local nodes_for_this_line
        do
            nodes_for_this_line = {}
            local max = 0
            local no = nil
            for i = 1, #snips do
                if founds[i].num ~= 0 and founds[i].hit ~= 0 and founds[i].hit ~=
                    founds[i].num then
                    if max < founds[i].hit / founds[i].num then
                        max = founds[i].hit / founds[i].num
                        no = i
                    end
                end
            end
            if no ~= nil then
                local s = snips[no]
                local n = founds[no]
                if n.hit < n.num_first then
                    nodes_for_this_line = s
                else
                    local j
                    local k = 0
                    for i = 2, #s do
                        j = i
                        if k >= n.hit - n.num_first then
                            break
                        end
                        if s[i].type == 'text' then
                            k = k + 1
                        end
                    end
                    nodes_for_this_line = copy(s, j, -1)
                end
            end
        end
        table.insert(res, nodes_for_this_line)
    end
    return res
end

local function text(node)
    if node.type == 'text' then return node.value end
    local res = ''
    for _, c in ipairs(node.children) do res = res .. text(c) end
    return res
end

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
    local matched = match(before_cursor_inclusive, world.snippets)
    local texts
    do
        texts = {}
        for l = world.start_line, world.cursor_line do
            local i = l - world.start_line + 1
            local nodes = matched[i]
            local chunks
            do
                local s = ''
                for _, n in ipairs(nodes) do s = s .. text(n) end
                chunks = {{s, world.highlight.base}}
            end
            table.insert(texts, {line = l, chunks = chunks})
        end
    end
    return {texts = texts}
end

return {update = update}
