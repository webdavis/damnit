local M = {}

local function unpushed_commits(doc)
  local commits = 0
  for _, remote in ipairs(doc.unpushed) do
    commits = commits + remote.commits
  end
  return commits
end

---@param doc damnit.StatusDoc
---@return string[][]
function M.chunks(doc)
  local commits = unpushed_commits(doc)
  local counts = {
    { #doc.conflicts, #doc.conflicts == 1 and "conflict" or "conflicts", "DamOverdue" },
    { #doc.waiting + #doc.sets, "waiting" },
    { #doc.unstaged, "unstaged" },
    { #doc.staged, "staged" },
    { commits, commits == 1 and "unpushed commit" or "unpushed commits" },
  }
  local chunks = { { " status", "DamHeader" }, { "", cut = true } }
  for _, count in ipairs(counts) do
    if count[1] > 0 then
      vim.list_extend(chunks, { { " · " }, { count[1] .. " " .. count[2], count[3] } })
    end
  end
  if #chunks == 2 then
    vim.list_extend(chunks, { { " · " }, { "clean" } })
  end
  return chunks
end

return M
