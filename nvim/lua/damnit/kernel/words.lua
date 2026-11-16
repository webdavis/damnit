local argv = require("damnit.kernel.argv")

local M = {}

---@class damnit.Flag
---@field name string
---@field value boolean
---@field help string

---@class damnit.Candidate
---@field word string
---@field menu string

---@class damnit.Category
---@field name string
---@field values string[]

---@class damnit.Vocab
---@field labels damnit.Candidate[]
---@field paths damnit.Candidate[]
---@field filters damnit.Candidate[]
---@field remotes damnit.Candidate[]
---@field categories damnit.Category[]
---@field rows? damnit.Candidate[] the tasks on screen, for flags that take an oid

local VALUES = {
  labels = "labels",
  unlabels = "labels",
  path = "paths",
  filter = "filters",
  remote = "remotes",
  parent = "rows",
  depends = "rows",
  undepends = "rows",
}

local function starting(prefix, candidates)
  return vim.tbl_filter(function(candidate)
    return vim.startswith(candidate.word, prefix)
  end, candidates)
end

local function flag_named(flags, name)
  for _, flag in ipairs(flags) do
    if flag.name == name then
      return flag
    end
  end
end

local function category_items(item, categories)
  local group = item:match("^([^=]*)=")
  local candidates = {}
  for _, category in ipairs(categories) do
    if not group then
      table.insert(candidates, { word = category.name .. "=", menu = table.concat(category.values, ", ") })
    elseif category.name == group then
      for _, value in ipairs(category.values) do
        table.insert(candidates, { word = group .. "=" .. value, menu = group })
      end
      table.insert(candidates, { word = group .. "=none", menu = ("takes %s's label off"):format(group) })
    end
  end
  return starting(item, candidates)
end

local function value_items(flag, item, vocab)
  local name = flag.name:sub(3)
  if name == "category" then
    return category_items(item, vocab.categories or {})
  end
  return starting(item, VALUES[name] and vocab[VALUES[name]] or {})
end

local function flag_items(current, flags)
  return starting(
    current,
    vim.tbl_map(function(flag)
      return { word = flag.name, menu = flag.help }
    end, flags)
  )
end

---@param line string
---@param col integer bytes before the cursor
---@param flags damnit.Flag[]
---@return boolean the word holding the cursor begins a flag name but is none of the command's flags
function M.unfinished(line, col, flags)
  for _, token in ipairs((argv.tokens(line))) do
    if token.from - 1 <= col and col <= token.from - 1 + #token.text then
      return #flags > 0 and vim.startswith(token.text, "--") and flag_named(flags, token.text) == nil
    end
  end
  return false
end

---@param line string
---@param col integer bytes before the cursor
---@param flags damnit.Flag[]
---@param vocab damnit.Vocab
---@return integer start, damnit.Candidate[] candidates
function M.at(line, col, flags, vocab)
  local before = line:sub(1, col)
  local tokens, open = argv.tokens(before)
  local current = open and tokens[#tokens] or { text = "", from = #before + 1 }
  local previous = tokens[#tokens - (open and 1 or 0)]
  local flag = previous and flag_named(flags, previous.text)
  if flag and flag.value and not vim.startswith(current.text, "--") then
    local comma = before:sub(current.from):match(".*(),") or 0
    local item = current.text:match("[^,]*$")
    return current.from - 1 + comma, value_items(flag, item, vocab)
  end
  if current.text == "" or vim.startswith(current.text, "-") then
    return current.from - 1, flag_items(current.text, flags)
  end
  return current.from - 1, {}
end

return M
