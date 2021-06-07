local function empty(table) return next(table) == nil end

-- Returns new snippets state from view state includes current buffer and cursor pos
-- world: {
--   lines: [str],
--   start_line: index,
--   cursor_line: index,
--   sources: [
--     source: [
--       snippets: [
--         snippet
--       ]
--     ]
--   ]
-- } # start_line <= cursor_line < start_line + len(lines)
-- index: int # zero-indexed
-- snippet: {
--   body: [str],
--   description: str,
--   label: str,
--   prefix: [str],
--   prefix_alias: [str],
-- }
-- return: {
--   texts: [
--     text: {
--       line: index,
--       chunks: [[text:str, hl_group:str]]
--     }
--   ]
-- }
local function update(world)
    if empty(world.sources) then return {texts = {}} end
    return {texts = {}}
end

return {update = update}
