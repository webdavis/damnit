local M = {}

local OP_GROUPS = { create = "DamAdded", update = "DamChanged", delete = "DamRemoved" }

---@class damnit.ConfirmRow
---@field op string
---@field subject string
---@field detail? string

local function widths_of(rows)
  local op, subject = 0, 0
  for _, row in ipairs(rows) do
    op = math.max(op, vim.fn.strdisplaywidth(row.op))
    subject = math.max(subject, vim.fn.strdisplaywidth(row.subject))
  end
  return op, subject
end

local function padded(text, width)
  return text .. (" "):rep(width - vim.fn.strdisplaywidth(text))
end

local function cut(text, width)
  if vim.fn.strdisplaywidth(text) <= width then
    return text
  end
  local chars = vim.fn.strchars(text)
  while chars > 0 and vim.fn.strdisplaywidth(vim.fn.strcharpart(text, 0, chars)) > width - 1 do
    chars = chars - 1
  end
  return vim.fn.strcharpart(text, 0, chars) .. "…"
end

---@param value any a field's value as dam prints it in JSON
---@return string
function M.value(value)
  if value == nil or value == "" or (type(value) == "table" and next(value) == nil) then
    return "none"
  end
  if type(value) == "boolean" then
    return value and "yes" or "no"
  end
  if type(value) ~= "table" then
    return tostring(value)
  end
  if vim.islist(value) then
    return table.concat(vim.tbl_map(M.value, value), ",")
  end
  return M.value(value.day or value.title or value.email or value.url)
end

---@param rows damnit.ConfirmRow[]
---@param summary string
---@param removes boolean
---@return string[] lines, damnit.Mark[] marks
function M.rows(rows, summary, removes)
  local lines, marks = {}, {}
  local op_width, subject_width = widths_of(rows)
  for n, row in ipairs(rows) do
    local detail = row.detail or ""
    local subject = detail == "" and row.subject or padded(row.subject, subject_width)
    local line = vim.trim(padded(row.op, op_width) .. "  " .. subject .. (detail == "" and "" or "  " .. detail))
    lines[n] = cut(line, vim.o.columns - 6)
    if OP_GROUPS[row.op] then
      table.insert(marks, { line = n - 1, col = 0, end_col = #row.op, group = OP_GROUPS[row.op] })
    end
  end
  if #lines > 0 then
    table.insert(lines, "")
  end
  table.insert(lines, summary)
  table.insert(marks, { line = #lines - 1, col = 0, end_col = #summary, group = removes and "DamRemoved" or "DamMeta" })
  return lines, marks
end

---@param answer { count: integer, changes: damnit.StatusChange[] }
---@param verb? string what the changes would do, "would change" when not given
---@return string[] lines, damnit.Mark[] marks
function M.changes(answer, verb)
  local removes = false
  local rows = vim.tbl_map(function(change)
    removes = removes or change.op == "delete"
    local afters = vim.tbl_map(function(field)
      return field .. " → " .. M.value(change[field])
    end, change.fields)
    return { op = change.op, subject = change.subject, detail = table.concat(afters, ", ") }
  end, answer.changes)
  return M.rows(rows, ("%d of %d %s"):format(#answer.changes, answer.count, verb or "would change"), removes)
end

---@param answer { remotes: { remote: string, mutations: table[], held: table[] }[] }
---@return string[] lines, damnit.Mark[] marks
function M.push(answer)
  local rows, sent, held = {}, 0, 0
  for _, remote in ipairs(answer.remotes) do
    for _, mutation in ipairs(remote.mutations) do
      sent = sent + 1
      table.insert(
        rows,
        { op = mutation.op, subject = mutation.subject or mutation.oid:sub(1, 7), detail = remote.remote }
      )
    end
    for _, kept in ipairs(remote.held) do
      held = held + 1
      table.insert(rows, { op = "held", subject = kept.oid:sub(1, 7) .. " " .. kept.why, detail = remote.remote })
    end
  end
  return M.rows(rows, ("%d would be sent, %d held"):format(sent, held), false)
end

return M
