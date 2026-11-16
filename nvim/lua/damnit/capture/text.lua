local M = {}

local MARKERS = { "TODO", "FIXME", "XXX", "NOTE" }

---@class damnit.CommentShape
---@field commentstring string
---@field comments string

local function longest_first(list)
  list = vim.tbl_filter(function(item)
    return item ~= ""
  end, list)
  table.sort(list, function(a, b)
    return #a > #b
  end)
  return list
end

local function ends_of(shape)
  local left, right = (shape.commentstring or ""):match("^(.-)%%s(.-)$")
  local leads, tails = { vim.trim(left or "") }, { vim.trim(right or "") }
  for _, entry in ipairs(vim.split(shape.comments or "", ",", { plain = true, trimempty = true })) do
    local flags, leader = entry:match("^([^:]*):(.*)$")
    if leader then
      table.insert(leads, vim.trim(leader))
      if flags:find("e", 1, true) then
        table.insert(tails, vim.trim(leader))
      end
    end
  end
  return longest_first(leads), longest_first(tails)
end

local function stripped(line, leads, tails)
  line = vim.trim(line)
  for _, lead in ipairs(leads) do
    if vim.startswith(line, lead) then
      line = vim.trim(line:sub(#lead + 1))
      break
    end
  end
  for _, tail in ipairs(tails) do
    if vim.endswith(line, tail) then
      line = vim.trim(line:sub(1, #line - #tail))
      break
    end
  end
  return line
end

local function unmarked(text)
  for _, marker in ipairs(MARKERS) do
    local rest = text:sub(#marker + 1)
    if vim.startswith(text, marker) and not rest:match("^%w") then
      return vim.trim((rest:gsub("^%b()", ""):gsub("^:", "")))
    end
  end
  return text
end

---@param lines string[]
---@param shape damnit.CommentShape
---@return string
function M.subject(lines, shape)
  local leads, tails = ends_of(shape)
  local kept = {}
  for _, line in ipairs(lines) do
    local text = stripped(line, leads, tails)
    if text ~= "" then
      table.insert(kept, text)
    end
  end
  return unmarked(table.concat(kept, " "))
end

---@param file string
---@param cwd string
---@param line integer
---@return string|nil
function M.place(file, cwd, line)
  if file == "" then
    return nil
  end
  return ("%s:%d"):format(vim.fs.relpath(cwd, file) or file, line)
end

---@param text string
---@return string
function M.quoted(text)
  return '"' .. text:gsub('[\\"]', "\\%0") .. '"'
end

---@param subject string
---@return string
function M.typed(subject)
  if subject:find("[\\\"']") or subject:find("%f[%S]%-%-") then
    return M.quoted(subject)
  end
  return subject
end

return M
