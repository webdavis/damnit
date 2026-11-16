local M = {}

local SCALARS = {
  { key = "subject" },
  { key = "due", clear = "--no-due" },
  { key = "deadline", clear = "--no-deadline" },
  { key = "priority" },
  { key = "estimate", clear = "--no-estimate" },
  { key = "recurrence", clear = "--no-recurrence" },
  { key = "start" },
  { key = "end" },
  { key = "location", clear = "--no-location" },
}

local function missing_from(items, others)
  local held = {}
  for _, item in ipairs(others) do
    held[item] = true
  end
  return vim.tbl_filter(function(item)
    return not held[item]
  end, items)
end

local function each_flag(flags, flag, items)
  for _, item in ipairs(items) do
    vim.list_extend(flags, { flag, item })
  end
end

local function scalar_flags(flags, before, after)
  for _, scalar in ipairs(SCALARS) do
    local was, now = before[scalar.key], after[scalar.key]
    if now ~= nil and now ~= was then
      if now == "" and scalar.clear then
        table.insert(flags, scalar.clear)
      else
        vim.list_extend(flags, { "--" .. scalar.key, now })
      end
    end
  end
end

local function category_flags(flags, before, after)
  local groups = vim.tbl_keys(vim.tbl_extend("force", {}, before, after))
  table.sort(groups)
  local taken_off = {}
  for _, group in ipairs(groups) do
    local was, now = before[group] or {}, after[group] or {}
    if #now == 0 then
      table.insert(flags, "--category")
      table.insert(flags, group .. "=none")
    else
      for _, label in ipairs(missing_from(now, was)) do
        vim.list_extend(flags, { "--category", group .. "=" .. label })
      end
      vim.list_extend(taken_off, missing_from(was, now))
    end
  end
  each_flag(flags, "--unlabels", taken_off)
end

---@param before damnit.Fields
---@param after damnit.Fields
---@return string[]
function M.diff(before, after)
  local flags = {}
  scalar_flags(flags, before, after)
  each_flag(flags, "--labels", missing_from(after.labels, before.labels))
  each_flag(flags, "--unlabels", missing_from(before.labels, after.labels))
  category_flags(flags, before.category, after.category)
  if after.body ~= before.body then
    vim.list_extend(flags, { "--body", after.body })
  end
  return flags
end

return M
