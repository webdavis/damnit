local Screen = {}
Screen.__index = Screen

local M = {}

---@return damnit.Screen
function M.new()
  return setmetatable({ namespace = vim.api.nvim_create_namespace("damnit") }, Screen)
end

local function named(name)
  for _, buf in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_get_name(buf) == name then
      return buf
    end
  end
end

---@param name string
---@param filetype string
---@return integer
function Screen:scratch(name, filetype)
  local buf = named(name)
  if buf then
    return buf
  end
  buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_name(buf, name)
  vim.bo[buf].bufhidden = "hide"
  vim.bo[buf].modifiable = false
  vim.bo[buf].filetype = filetype
  return buf
end

---@param buf integer
---@param lines string[]
---@param marks damnit.Mark[]
function Screen:draw(buf, lines, marks)
  vim.bo[buf].modifiable = true
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.bo[buf].modifiable = false
  vim.bo[buf].modified = false
  vim.api.nvim_buf_clear_namespace(buf, self.namespace, 0, -1)
  for _, mark in ipairs(marks) do
    vim.api.nvim_buf_set_extmark(
      buf,
      self.namespace,
      mark.line,
      mark.col,
      { end_col = mark.end_col, hl_group = mark.group }
    )
  end
end

local function splits(smods)
  return smods ~= nil and (smods.vertical or smods.horizontal or (smods.split or "") ~= "" or (smods.tab or -1) >= 0)
end

---@param buf integer
---@param smods? table
function Screen:show(buf, smods)
  if splits(smods) then
    vim.cmd.sbuffer({ args = { buf }, mods = smods })
  else
    vim.api.nvim_win_set_buf(0, buf)
  end
end

return M
