local M = {}

---@param commit damnit.Commit
---@param oid string
---@return string
local function named(commit, oid)
  for _, change in ipairs(commit.changes) do
    if change.oid == oid then
      return change.subject
    end
  end
  return oid:sub(1, 7)
end

---@param answer damnit.RevertAnswer
---@param commit damnit.Commit the commit reverted, as dam show printed it
---@return string
function M.text(answer, commit)
  local count = #answer.reverted
  local text = ("%d change%s came back"):format(count, count == 1 and "" or "s")
  if #answer.kept == 0 then
    return text
  end
  local kept = vim.tbl_map(function(row)
    return named(commit, row.oid) .. " " .. table.concat(row.fields, ", ")
  end, answer.kept)
  return text .. "; kept as changed since: " .. table.concat(kept, "; ")
end

return M
