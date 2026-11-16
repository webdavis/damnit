local align = require("damnit.kernel.align")
local columns = require("damnit.status.columns")

local M = {}

---@class damnit.StatusTarget
---@field kind "section"|"change"|"waiting"|"set"|"conflict"|"remote"|"note"
---@field section string
---@field id? string
---@field oid? string

local OP_GROUPS = { create = "DamAdded", update = "DamChanged", delete = "DamRemoved" }

---@param text string
---@param target damnit.StatusTarget
---@return damnit.StatusRow
local function plain(text, target)
  return { left = { { text } }, target = target }
end

local function changes(section)
  return function(doc)
    return vim.tbl_map(function(change)
      local fields = change.op == "create" and "" or table.concat(change.fields, ", ")
      return {
        shape = "change",
        left = { { change.op }, { change.subject } },
        right = { { change.path }, { fields, "DamMeta" } },
        target = { kind = "change", section = section, id = change.oid, oid = change.oid },
        group = OP_GROUPS[change.op],
      }
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
    return plain(text, { kind = "conflict", section = "conflicts", id = conflict.oid, oid = conflict.oid })
  end, doc.conflicts)
end

local function sent(what, subject, from, age, target)
  return {
    shape = "waiting",
    left = { { what }, { subject } },
    right = { { "from " .. from }, { columns.ago(age), "DamMeta" } },
    target = target,
  }
end

local function waiting(doc)
  local rows = vim.tbl_map(function(item)
    return sent(item.item, item.subject, item.from, item.age, {
      kind = "waiting",
      section = "waiting",
      id = item.hash,
      oid = item.oid,
    })
  end, doc.waiting)
  for _, set in ipairs(doc.sets) do
    local target = { kind = "set", section = "waiting", id = set.set }
    table.insert(rows, sent(("set of %d"):format(set.count), "", set.from, set.age, target))
  end
  return rows
end

local function unpushed(doc)
  local rows = {}
  for _, remote in ipairs(doc.unpushed) do
    if remote.commits > 0 then
      local text = ("%s  %d commit%s"):format(remote.remote, remote.commits, remote.commits == 1 and "" or "s")
      table.insert(rows, plain(text, { kind = "remote", section = "unpushed", id = remote.remote }))
    end
  end
  return rows
end

local function notes(doc)
  local rows = vim.tbl_map(function(notice)
    local text = ("%s: %s"):format(notice.remote or "dam", (notice.kind:gsub("_", " ")))
    return plain(text, { kind = "note", section = "notices" })
  end, doc.notices)
  return rows
end

local function warnings(doc)
  return vim.tbl_map(function(warning)
    return plain(warning, { kind = "note", section = "warnings" })
  end, doc.warnings)
end

local SECTIONS = {
  { "conflicts", "Conflicts", conflicts },
  { "waiting", "Waiting", waiting },
  { "unstaged", "Unstaged", changes("unstaged") },
  { "staged", "Staged", changes("staged") },
  { "unpushed", "Unpushed", unpushed },
  { "notices", "Notices", notes },
  { "warnings", "Warnings", warnings },
}

local function sections_of(doc, width)
  local shown, all = {}, {}
  for _, section in ipairs(SECTIONS) do
    local rows = section[3](doc)
    if #rows > 0 then
      table.insert(shown, { name = section[1], title = section[2], rows = rows })
      vim.list_extend(all, rows)
    end
  end
  return shown, columns.laid(all, width)
end

---@param doc damnit.StatusDoc
---@param width? integer the window's text width
---@return string[] lines, damnit.Mark[] marks, table<integer, damnit.StatusTarget> targets
function M.render(doc, width)
  local lines, marks, targets = {}, {}, {}
  width = width or math.huge
  local shown, laid = sections_of(doc, width)
  for _, section in ipairs(shown) do
    if #lines > 0 then
      table.insert(lines, "")
    end
    local header = ("%s (%d)"):format(section.title, #section.rows)
    table.insert(lines, header)
    table.insert(marks, { line = #lines - 1, col = 0, end_col = #header, group = "DamHeader" })
    targets[#lines] = { kind = "section", section = section.name }
    for _, row in ipairs(section.rows) do
      local text, chunks = laid(row)
      table.insert(lines, "  " .. text)
      targets[#lines] = row.target
      if row.group then
        table.insert(marks, { line = #lines - 1, col = 2, end_col = #lines[#lines], group = row.group })
      end
      local mark = align.right(#lines - 1, vim.fn.strdisplaywidth(lines[#lines]), chunks, width)
      table.insert(marks, mark)
    end
  end
  if #lines == 0 then
    return { "nothing changed, staged, waiting or unpushed" }, {}, {}
  end
  return lines, marks, targets
end

return M
