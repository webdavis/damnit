local actions = require("damnit.list.actions")
local marked = require("damnit.list.marked")
local repeating = require("damnit.kernel.repeat")
local targets = require("damnit.list.targets")

local ChangeKeys = {}

local function visual_range()
  if not vim.fn.mode():match("^[vV\22]") then
    return nil
  end
  local range = { vim.fn.line("v"), vim.fn.line(".") }
  vim.cmd.normal({ vim.keycode("<Esc>"), bang = true })
  return range
end

---@param buf integer
---@param count integer
---@param range? integer[]
---@param skip_marks? boolean
---@return string[]
function ChangeKeys:targets(buf, count, range, skip_marks)
  local list = self.shown[buf]
  return targets.oids(list.oids, {
    row = vim.api.nvim_win_get_cursor(0)[1],
    count = count,
    range = range,
    marked = skip_marks and {} or (list.marked or {}),
    lines = vim.api.nvim_buf_line_count(buf),
  })
end

---@param buf integer
---@param key string
function ChangeKeys:press(buf, key)
  local count = vim.v.count
  local oids = self:targets(buf, count, visual_range())
  if #oids == 0 then
    return self.app.say:warn(key .. " acts on task rows; this line holds no task")
  end
  local answer, handle
  handle = repeating.start(function()
    self:act(buf, key, oids, nil, {
      answered = function(text)
        answer = text
      end,
      closed = function()
        handle.rearm()
      end,
    })
  end, function(again)
    local here = vim.api.nvim_get_current_buf()
    if not self.shown[here] or (actions.prompt(key) and not answer) then
      return
    end
    local more = self:targets(here, again > 0 and again or count)
    if #more > 0 then
      self:act(here, key, more, answer)
    end
  end)
end

---@param buf integer
function ChangeKeys:mark(buf)
  local list = self.shown[buf]
  list.marked = marked.toggled(list.marked or {}, self:targets(buf, vim.v.count, visual_range(), true))
  self:draw(buf)
end

---@param buf integer
---@return damnit.KeySpec[]
function ChangeKeys:change_keys(buf)
  local specs = {}
  for _, key in ipairs(actions.keys()) do
    local spec = actions.spec(key)
    table.insert(specs, {
      lhs = key,
      desc = spec.desc,
      group = spec.group,
      mode = key == "dd" and "n" or { "n", "x" },
      run = function()
        self:press(buf, key)
      end,
    })
  end
  local function call(lhs, mode, desc, run)
    table.insert(specs, { lhs = lhs, desc = desc, group = "task rows", mode = mode, run = run })
  end
  call("d", "x", "remove the selected rows, after the confirm float", function()
    self:press(buf, "dd")
  end)
  call("m", { "n", "x" }, "mark or unmark", function()
    self:mark(buf)
  end)
  call("M", "n", "unmark all", function()
    self.shown[buf].marked = {}
    self:draw(buf)
  end)
  return specs
end

return ChangeKeys
