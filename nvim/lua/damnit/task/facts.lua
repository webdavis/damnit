local M = {}

local FLAGGED = { { "overdue", "DamOverdue" }, { "blocked", "DamBlocked" } }

---@param object damnit.Object
---@return string[][]
function M.chunks(object)
  local parts = { { " " .. object.oid:sub(1, 7), "DamMeta" } }
  if (object.path or "") ~= "" then
    table.insert(parts, { object.path })
  end
  if object.author then
    table.insert(parts, { "by " .. object.author })
  end
  if object.parent then
    table.insert(parts, { "parent " .. object.parent:sub(1, 7) })
  end
  for _, flag in ipairs(FLAGGED) do
    if object[flag[1]] then
      table.insert(parts, flag)
    end
  end
  local kept = (object.path or "") ~= "" and 2 or 1
  local facts = {}
  for n, part in ipairs(parts) do
    if n > 1 then
      table.insert(facts, { " · " })
    end
    table.insert(facts, part)
    if n == kept then
      table.insert(facts, { "", cut = true })
    end
  end
  return facts
end

return M
