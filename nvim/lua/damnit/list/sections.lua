local groups = require("damnit.list.groups")
local tree = require("damnit.list.tree")

local M = {}

---@class damnit.Segment
---@field title string
---@field objects? damnit.Object[]
---@field rows? damnit.Row[] rows already arranged, drawn as they are
---@field uncounted? boolean the header reads the title alone
---@field key? string names the header from one draw to the next; the title when left out
---@field error? damnit.Error

---@param segment damnit.Segment
---@return damnit.Row
local function header_of(segment)
  if segment.error then
    return { header = segment.title .. ": " .. segment.error.message, failed = true, depth = 0, key = segment.title }
  end
  local header = segment.uncounted and segment.title
    or ("%s (%d)"):format(segment.title, #(segment.rows or segment.objects))
  return { header = header, depth = 0, key = segment.key or segment.title }
end

---@param segments damnit.Segment[]
---@return damnit.Row[]
function M.rows(segments)
  local rows = {}
  for _, segment in ipairs(segments) do
    table.insert(rows, header_of(segment))
    vim.list_extend(rows, segment.error and {} or segment.rows or tree.arranged(segment.objects))
  end
  return rows
end

---@param parts damnit.ListPart[]
---@param answers { answer?: damnit.ListAnswer, err?: damnit.Error }[]
---@param grouped_by? string|string[]
---@return damnit.Row[]|nil rows, damnit.Error|nil err the refusal of a list of one part
function M.arranged(parts, answers, grouped_by)
  if #parts == 1 then
    local got = answers[1]
    if got.err then
      return nil, got.err
    end
    local segments = groups.segments(got.answer.objects, grouped_by)
    return segments and M.rows(segments) or tree.arranged(got.answer.objects), nil
  end
  local segments = {}
  for n, part in ipairs(parts) do
    local got = answers[n]
    segments[n] = { title = part.title or "", objects = got.answer and got.answer.objects, error = got.err }
  end
  return M.rows(segments), nil
end

return M
