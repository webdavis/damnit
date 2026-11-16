local argv = require("damnit.kernel.argv")
local view = require("damnit.list.view")
local words = require("damnit.kernel.words")

local FilterBar = {}

local SECTIONS = "one list in place of the sections"

---@param count integer
---@return string
local function rows(count)
  return ("%d row%s"):format(count, count == 1 and "" or "s")
end

---@param vocab damnit.Vocab
---@return table<string, table>
local function saved_tables(vocab)
  local tables = {}
  for _, filter in ipairs(vocab.filters or {}) do
    tables[filter.word] = filter.table
  end
  return tables
end

---@param flags damnit.Flag[]
---@param refused { flag?: string, shapes?: string[] }
---@return damnit.Flag[]
local function with_shapes(flags, refused)
  if not refused.flag then
    return flags
  end
  return vim.tbl_map(function(flag)
    return flag.name == refused.flag and vim.tbl_extend("force", flag, { values = refused.shapes }) or flag
  end, flags)
end

---@param buf integer
---@return damnit.KeySpec[]
function FilterBar:filter_keys(buf)
  local run = function()
    self:filter(buf)
  end
  return { { lhs = "f", desc = "filter bar: change what the list selects", group = "task rows", run = run } }
end

---@param list table
---@return string|nil the view's conditions as flags, nil for a view of sections
function FilterBar:conditions(list)
  if list.filter_text then
    return list.filter_text
  end
  local vocab = self.app.vocab:words(list.call.cwd)
  vim.wait(self.app.options.timeout_ms, function()
    return vocab.filters ~= nil
  end, 10)
  return view.conditions(list.view, self.app.options.views, saved_tables(vocab))
end

---@param buf integer
---@param flags damnit.Flag[]
---@param refused table
---@param note? string
function FilterBar:filter_read(buf, flags, refused, note)
  local list = self.shown[buf]
  return function(text, done)
    local args = vim.list_extend({ "list" }, argv.args(text, flags))
    return self.app.dam:run({ args = args, cwd = list.call.cwd, key = list.call.key }, function(answer, err)
      if err then
        refused.flag = err.usage and err.usage.word
        refused.shapes = err.selection and err.selection.shapes
        return done(err.message, err)
      end
      refused.flag = nil
      list.parts, list.grouped_by = { { args = args } }, nil
      self:arrange(buf, { { answer = answer } })
      done(table.concat({ rows(#answer.objects), note }, " · "), false)
    end)
  end
end

---@param buf integer
function FilterBar:filter(buf)
  local list = self.shown[buf]
  local before = { parts = list.parts, grouped_by = list.grouped_by, name = list.name, text = list.filter_text }
  local conditions = self:conditions(list)
  local flags, refused = self.app.vocab:flags("list"), {}
  local function restore()
    list.parts, list.grouped_by, list.name, list.filter_text = before.parts, before.grouped_by, before.name, before.text
    self:read(buf)
  end
  self.prompts = self.prompts or require("damnit.kernel.prompt").new(self.app.queue, self.app.say)
  self.prompts:open({
    title = "dam list",
    prefill = conditions or "",
    anchor = { top = vim.api.nvim_get_current_win() },
    read_first = true,
    reads_blank = true,
    flags = flags,
    read = self:filter_read(buf, flags, refused, conditions == nil and SECTIONS or nil),
    submit = function(text)
      if text == (conditions or "") and not before.text then
        return restore()
      end
      list.filter_text, list.name = text, before.name:gsub("%*$", "") .. "*"
      self:draw(buf)
    end,
    closed = function(sent)
      if not sent then
        restore()
      end
    end,
    complete = function(line, col)
      return words.at(line, col, with_shapes(flags, refused), self.app.vocab:words(list.call.cwd))
    end,
  })
end

return FilterBar
