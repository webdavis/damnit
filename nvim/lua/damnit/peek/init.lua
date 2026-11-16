local card = require("damnit.peek.card")
local float = require("damnit.kernel.float")

local MOST = 80

local Peek = {}
Peek.__index = Peek

local M = {}

---@class damnit.Peek
---@field peek fun(oid: string|nil, cwd: string)
---@field again fun(): boolean
---@field key fun(locate: fun(): string|nil, string): damnit.KeySpec

function Peek:close()
  local shown = self.shown
  self.shown = nil
  if not shown then
    return
  end
  pcall(vim.api.nvim_del_augroup_by_id, shown.group)
  if vim.api.nvim_win_is_valid(shown.win) then
    vim.api.nvim_win_close(shown.win, true)
  end
  if vim.api.nvim_win_is_valid(shown.source) then
    vim.api.nvim_set_current_win(shown.source)
  end
end

function Peek:bind(shown, oid, cwd)
  local function close()
    self:close()
  end
  for _, lhs in ipairs({ "q", "<Esc>" }) do
    vim.keymap.set("n", lhs, close, { buffer = shown.buf, nowait = true, desc = "close the peek" })
  end
  vim.keymap.set("n", "<CR>", function()
    self:close()
    self.open_task(oid, cwd)
  end, { buffer = shown.buf, nowait = true, desc = "open the task" })
  local source_buf = vim.api.nvim_win_get_buf(shown.source)
  vim.api.nvim_create_autocmd({ "CursorMoved", "BufWinLeave" }, {
    group = shown.group,
    buffer = source_buf,
    callback = function()
      if self.shown == shown and vim.api.nvim_get_current_win() == shown.source then
        close()
      end
    end,
  })
  vim.api.nvim_create_autocmd("WinLeave", { group = shown.group, buffer = shown.buf, callback = close })
end

---@param object damnit.Object
---@param cwd string
function Peek:show(object, oid, cwd)
  self:close()
  local source = vim.api.nvim_get_current_win()
  local textoff = vim.fn.getwininfo(source)[1].textoff
  local room = math.max(math.min(MOST, vim.api.nvim_win_get_width(source) - textoff - 2), 10)
  local shown_card = card.render(object, room)
  local buf = vim.api.nvim_create_buf(false, true)
  vim.bo[buf].bufhidden = "wipe"
  self.app.screen:draw(buf, shown_card.lines, shown_card.marks)
  local win = float.card(buf, shown_card)
  local shown = { win = win, buf = buf, source = source }
  shown.group = vim.api.nvim_create_augroup("damnit.peek." .. buf, { clear = true })
  self.shown = shown
  self:bind(shown, oid, cwd)
end

---@param oid string|nil
---@param cwd string
function Peek:peek(oid, cwd)
  if not oid then
    return self.app.say:warn("no task or event on this line to peek at")
  end
  local source, cursor = vim.api.nvim_get_current_win(), vim.api.nvim_win_get_cursor(0)
  self.app.dam:run({ args = { "show", oid }, cwd = cwd, key = "peek" }, function(answer, err)
    if err then
      return self.app.say:error(err)
    end
    local here = vim.api.nvim_get_current_win() == source
    if here and vim.deep_equal(vim.api.nvim_win_get_cursor(0), cursor) then
      self:show(answer, oid, cwd)
    end
  end)
end

---@return boolean entered
function Peek:again()
  local shown = self.shown
  if shown and vim.api.nvim_win_is_valid(shown.win) and vim.api.nvim_get_current_win() == shown.source then
    vim.api.nvim_set_current_win(shown.win)
    return true
  end
  return false
end

---@param app damnit.App
---@param open_task fun(oid: string, cwd: string)
---@return damnit.Peek
function M.new(app, open_task)
  local peek = setmetatable({ app = app, open_task = open_task }, Peek)
  return {
    peek = function(oid, cwd)
      peek:peek(oid, cwd)
    end,
    again = function()
      return peek:again()
    end,
    key = function(locate)
      return {
        lhs = "K",
        desc = "peek at the task, or enter the peek",
        run = function()
          if not peek:again() then
            peek:peek(locate())
          end
        end,
      }
    end,
  }
end

return M
