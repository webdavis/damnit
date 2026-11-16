local M = {}

---@class damnit.Counts
---@field overdue integer
---@field waiting integer
---@field unstaged integer
---@field staged integer
---@field unpushed integer

local SHOWN = {
  { count = "overdue", sign = "overdue", group = "DamOverdue" },
  { count = "waiting", sign = "waiting", group = "DamMeta" },
  { count = "unstaged", sign = "changed", group = "DamChanged" },
  { count = "staged", sign = "staged", group = "DamAdded" },
  { count = "unpushed", text = "^", group = "DamMeta" },
}

---@param overdue damnit.ListAnswer
---@param status damnit.StatusDoc
---@return damnit.Counts
function M.counts(overdue, status)
  local unpushed = 0
  for _, remote in ipairs(status.unpushed) do
    unpushed = unpushed + remote.commits
  end
  return {
    overdue = #overdue.objects,
    waiting = #status.waiting + #status.sets,
    unstaged = #status.unstaged,
    staged = #status.staged,
    unpushed = unpushed,
  }
end

---@param counts damnit.Counts|nil
---@param signs table<string, string>
---@param highlight? boolean
---@return string
function M.text(counts, signs, highlight)
  local parts = {}
  for _, shown in ipairs(SHOWN) do
    local count = counts and counts[shown.count] or 0
    if count > 0 then
      local part = ((shown.text or signs[shown.sign]) .. count):gsub("%%", "%%%%")
      table.insert(parts, highlight and ("%#" .. shown.group .. "#" .. part .. "%*") or part)
    end
  end
  return table.concat(parts, " ")
end

return M
