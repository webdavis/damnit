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

---@param mark damnit.Mark
---@return vim.api.keyset.set_extmark
local function extmark_options(mark)
  return {
    end_col = mark.end_col,
    hl_group = mark.group,
    virt_text = mark.virt,
    virt_text_pos = mark.align,
    sign_text = mark.sign,
    sign_hl_group = mark.sign_group,
    line_hl_group = mark.line_group,
    virt_lines = mark.virt_lines,
  }
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
    vim.api.nvim_buf_set_extmark(buf, self.namespace, mark.line, mark.col or 0, extmark_options(mark))
  end
end

---@param chunks string[][]
---@return string
function M.bar(chunks)
  return table.concat(vim.tbl_map(function(chunk)
    if chunk.cut then
      return "%<"
    end
    local text = chunk[1]:gsub("%%", "%%%%")
    return chunk[2] and ("%#" .. chunk[2] .. "#" .. text .. "%*") or text
  end, chunks))
end

local function look_now(buf)
  for _, win in ipairs(vim.fn.win_findbuf(buf)) do
    for name, value in pairs(vim.b[buf].damnit_window or {}) do
      vim.wo[win][0][name] = value
    end
  end
end

---@param buf integer
---@param options table<string, string|boolean>
function Screen:window(buf, options)
  if not vim.b[buf].damnit_window then
    vim.api.nvim_create_autocmd("BufWinEnter", {
      buffer = buf,
      callback = function()
        look_now(buf)
      end,
    })
  end
  vim.b[buf].damnit_window = options
  look_now(buf)
end

---@param buf integer
---@return integer
function Screen:text_width(buf)
  local widths = vim.tbl_map(function(win)
    local info = vim.fn.getwininfo(win)[1]
    return info.width - info.textoff
  end, vim.fn.win_findbuf(buf))
  return #widths > 0 and math.min(unpack(widths)) or vim.o.columns
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
