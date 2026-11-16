local float = require("damnit.kernel.float")
local history = require("damnit.kernel.history")

local Session = {}
Session.__index = Session

local M = {}

---@class damnit.PromptSession
---@field win integer
---@field buf integer
---@field spec damnit.PromptSpec
---@field judge damnit.Judge

---@param fields { win: integer, buf: integer, spec: damnit.PromptSpec, entries: string[], judge: damnit.Judge }
---@return damnit.PromptSession
function M.new(fields)
  fields.browse = history.browse(fields.entries)
  return setmetatable(fields, Session)
end

---@param before string
---@param after? string
function Session:fill(before, after)
  self.prefilling = after ~= nil
  vim.api.nvim_buf_set_lines(self.buf, 0, -1, false, { before .. (after or "") })
  self.prefilling = false
  vim.api.nvim_win_set_cursor(self.win, { 1, #before })
end

function Session:prefill()
  self:fill(self.spec.prefill, self.spec.after or "")
end

---@return string
function Session:text()
  return table.concat(vim.api.nvim_buf_get_lines(self.buf, 0, -1, false), " ")
end

function Session:close()
  if self.closed then
    return
  end
  self.closed = true
  self.judge:close()
  if vim.api.nvim_win_is_valid(self.win) then
    vim.api.nvim_win_close(self.win, true)
  end
  vim.cmd.stopinsert()
end

function Session:clear()
  self.judge:stop()
  float.footer(self.win, "", "FloatFooter")
end

function Session:changed()
  if self.closed then
    return
  end
  local text = self:text()
  if vim.trim(text) == "" then
    return self:clear()
  end
  self.judge:soon(text)
end

---@param stay boolean
function Session:submit(stay)
  local text = self:text()
  if vim.trim(text) == "" then
    return stay or self:close()
  end
  self.judge:now(text, function(refused)
    if refused or text ~= self:text() then
      return
    end
    history.kept(self.entries, text)
    self.browse = history.browse(self.entries)
    if stay then
      self:clear()
      self:prefill()
    else
      self:close()
    end
    self.spec.submit(text)
  end)
end

function Session:older()
  local text = self.browse:older(self:text())
  if text then
    self:fill(text)
  end
end

function Session:newer()
  local text = self.browse:newer()
  if text then
    self:fill(text)
  end
end

return M
