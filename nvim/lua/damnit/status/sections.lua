local M = {}

---@class damnit.StatusTarget
---@field kind "section"|"change"|"waiting"|"set"|"conflict"|"remote"|"note"
---@field section string
---@field id? string
---@field oid? string

---@param change damnit.StatusChange
---@return string
local function change_text(change)
  local words = { change.op .. " " .. change.subject }
  if change.path ~= "" then
    table.insert(words, change.path)
  end
  if #change.fields > 0 then
    table.insert(words, "[" .. table.concat(change.fields, ", ") .. "]")
  end
  return table.concat(words, "  ")
end

local function changes(section)
  return function(doc)
    return vim.tbl_map(function(change)
      return { change_text(change), { kind = "change", section = section, id = change.oid, oid = change.oid } }
    end, doc[section])
  end
end

local function conflicts(doc)
  return vim.tbl_map(function(conflict)
    local text = ("%s  %s  run dam resolve %s --ours or --theirs in a terminal"):format(
      conflict.ours and conflict.ours.subject or conflict.oid:sub(1, 7),
      conflict.remote,
      conflict.oid:sub(1, 7)
    )
    return { text, { kind = "conflict", section = "conflicts", id = conflict.oid, oid = conflict.oid } }
  end, doc.conflicts)
end

local function waiting(doc)
  local rows = vim.tbl_map(function(item)
    local text = ("%s %s  from %s"):format(item.item, item.subject, item.from)
    return { text, { kind = "waiting", section = "waiting", id = item.hash, oid = item.oid } }
  end, doc.waiting)
  for _, set in ipairs(doc.sets) do
    table.insert(
      rows,
      { ("set of %d  from %s"):format(set.count, set.from), { kind = "set", section = "waiting", id = set.set } }
    )
  end
  return rows
end

local function unpushed(doc)
  local rows = {}
  for _, remote in ipairs(doc.unpushed) do
    if remote.commits > 0 then
      local text = ("%s  %d commit%s"):format(remote.remote, remote.commits, remote.commits == 1 and "" or "s")
      table.insert(rows, { text, { kind = "remote", section = "unpushed", id = remote.remote } })
    end
  end
  return rows
end

local function notices(doc)
  return vim.tbl_map(function(notice)
    return {
      ("%s: %s"):format(notice.remote or "dam", (notice.kind:gsub("_", " "))),
      { kind = "note", section = "notices" },
    }
  end, doc.notices)
end

local function warnings(doc)
  return vim.tbl_map(function(warning)
    return { warning, { kind = "note", section = "warnings" } }
  end, doc.warnings)
end

local SECTIONS = {
  { "conflicts", "Conflicts", conflicts },
  { "waiting", "Waiting", waiting },
  { "unstaged", "Unstaged", changes("unstaged") },
  { "staged", "Staged", changes("staged") },
  { "unpushed", "Unpushed", unpushed },
  { "notices", "Notices", notices },
  { "warnings", "Warnings", warnings },
}

---@param doc damnit.StatusDoc
---@return string[] lines, damnit.Mark[] marks, table<integer, damnit.StatusTarget> targets
function M.render(doc)
  local lines, marks, targets = {}, {}, {}
  for _, section in ipairs(SECTIONS) do
    local name, title, rows_of = unpack(section)
    local rows = rows_of(doc)
    if #rows > 0 then
      if #lines > 0 then
        table.insert(lines, "")
      end
      local header = ("%s (%d)"):format(title, #rows)
      table.insert(lines, header)
      table.insert(marks, { line = #lines - 1, col = 0, end_col = #header, group = "DamSection" })
      targets[#lines] = { kind = "section", section = name }
      for _, row in ipairs(rows) do
        table.insert(lines, "  " .. row[1])
        targets[#lines] = row[2]
      end
    end
  end
  if #lines == 0 then
    return { "nothing changed, staged, waiting or unpushed" }, {}, {}
  end
  return lines, marks, targets
end

return M
