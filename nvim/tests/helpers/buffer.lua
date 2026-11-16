local M = {}

---@param buf? integer
---@return string[]
function M.lines(buf)
  return vim.api.nvim_buf_get_lines(buf or 0, 0, -1, false)
end

---@param keys string
function M.feed(keys)
  vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes(keys, true, false, true), "x", false)
end

---@param check fun():boolean
---@param ms? integer
function M.wait_for(check, ms)
  assert(vim.wait(ms or 2000, check, 5), ("waited %d ms and it never happened"):format(ms or 2000))
end

---@return string[] notes
function M.capture_notes()
  local notes = {}
  vim.notify = function(text, level)
    table.insert(notes, { text = text, level = level })
  end
  return notes
end

return M
