local M = {}

---@class damnit.Fields
---@field labels string[]
---@field category table<string, string[]>
---@field body string

local TASK_KEYS = { "subject", "due", "deadline", "priority", "labels", "category", "estimate", "recurrence" }
local EVENT_KEYS = { "subject", "start", "end", "location", "labels", "category" }
local DERIVED = { "oid", "path", "parent", "author" }
local FLAGGED = { "overdue", "blocked" }
local RULE = "---"

local function values_of(held)
  return type(held) == "table" and held or { held }
end

local function category_words(category)
  local groups = vim.tbl_keys(category or {})
  table.sort(groups)
  local words = {}
  for _, group in ipairs(groups) do
    for _, label in ipairs(values_of(category[group])) do
      table.insert(words, group .. "=" .. label)
    end
  end
  return words
end

local function free_labels(object)
  local in_category = {}
  for _, held in pairs(object.category or {}) do
    for _, label in ipairs(values_of(held)) do
      in_category[label] = true
    end
  end
  return vim.tbl_filter(function(label)
    return not in_category[label]
  end, object.labels or {})
end

local function texts_of(object)
  local task, event = object.task or {}, object.event or {}
  return {
    subject = object.subject,
    due = task.due,
    deadline = task.deadline,
    priority = task.priority,
    estimate = task.estimate,
    recurrence = object.recurrence,
    start = event.start,
    ["end"] = event["end"],
    location = event.location,
    labels = table.concat(free_labels(object), ", "),
    category = table.concat(category_words(object.category), ", "),
  }
end

local function virtual_lines(object)
  local virt = {}
  for _, key in ipairs(DERIVED) do
    if object[key] then
      table.insert(virt, key .. ": " .. object[key])
    end
  end
  for _, key in ipairs(FLAGGED) do
    if object[key] then
      table.insert(virt, key .. ": true")
    end
  end
  return virt
end

---@param object damnit.Object
---@return string[] lines, string[] virt
function M.lines(object)
  local texts = texts_of(object)
  local lines = {}
  for _, key in ipairs(object.kind == "event" and EVENT_KEYS or TASK_KEYS) do
    local text = tostring(texts[key] or "")
    table.insert(lines, key .. ":" .. (text ~= "" and " " .. text or ""))
  end
  table.insert(lines, RULE)
  if (object.body or "") ~= "" then
    vim.list_extend(lines, vim.split(object.body, "\n", { plain = true }))
  end
  return lines, virtual_lines(object)
end

local function list_of(text)
  local items = {}
  for _, item in ipairs(vim.split(text, ",", { plain = true })) do
    item = vim.trim(item)
    if item ~= "" then
      table.insert(items, item)
    end
  end
  return items
end

local function category_of(text, n)
  local category = {}
  for _, word in ipairs(list_of(text)) do
    local group, label = word:match("^([^=%s]+)%s*=%s*([^=%s]+)$")
    if not group then
      return nil, ("line %d: category %q is not group=label, such as effort=deep"):format(n, word)
    end
    category[group] = category[group] or {}
    table.insert(category[group], label)
  end
  return category
end

local function keys_for(given)
  return given.start ~= nil and EVENT_KEYS or TASK_KEYS
end

local function field_lines(lines)
  local given, known = {}, {}
  for _, key in ipairs(vim.list_extend(vim.list_slice(TASK_KEYS), EVENT_KEYS)) do
    known[key] = true
  end
  for n, line in ipairs(lines) do
    if line == RULE then
      return given, n
    end
    local key, value = line:match("^(%a+):(.*)$")
    if not known[key] then
      return nil,
        ("line %d: %q is not a header field; write key: value with a key of %s"):format(
          n,
          line,
          table.concat(keys_for(given), ", ")
        )
    end
    if given[key] then
      return nil, ("line %d: %s: is given twice; keep one"):format(n, key)
    end
    given[key] = { text = vim.trim(value), line = n }
  end
  return nil, ("the header has no %s line under it; put %s between the header and the body"):format(RULE, RULE)
end

---@param lines string[]
---@return damnit.Fields|nil fields, string|nil problem
function M.parse(lines)
  local given, rule = field_lines(lines)
  if not given then
    return nil, rule
  end
  local fields = { body = table.concat(vim.list_slice(lines, rule + 1), "\n") }
  for _, key in ipairs(keys_for(given)) do
    if not given[key] then
      return nil, ("the header needs a %s: line; leave its value empty to clear it"):format(key)
    end
    fields[key] = given[key].text
  end
  fields.labels = list_of(fields.labels)
  local category, problem = category_of(fields.category, given.category.line)
  if not category then
    return nil, problem
  end
  fields.category = category
  return fields
end

return M
