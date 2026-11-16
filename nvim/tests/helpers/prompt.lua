local M = {}

---@param keys string
function M.type(keys)
  vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes(keys, true, false, true), "tx", false)
end

---@param keys string
function M.more(keys)
  M.type("A" .. keys)
end

---@param open fun()
---@param keys? string
function M.open(open, keys)
  vim.keymap.set("n", "<Plug>(dam-spec-open)", open)
  M.type("<Plug>(dam-spec-open)" .. (keys or ""))
end

---@return integer|nil
function M.window()
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.bo[vim.api.nvim_win_get_buf(win)].filetype == "damprompt" then
      return win
    end
  end
end

---@return string text, string|nil group
function M.footer()
  local win = assert(M.window(), "no prompt is open")
  local footer = vim.api.nvim_win_get_config(win).footer or {}
  local chunk = footer[1] or {}
  return vim.trim(chunk[1] or ""), chunk[2]
end

---@return string
function M.line()
  local win = assert(M.window(), "no prompt is open")
  return vim.api.nvim_buf_get_lines(vim.api.nvim_win_get_buf(win), 0, -1, false)[1]
end

---@return string[][][] each line under the input, as chunks
function M.below_chunks()
  local win = assert(M.window(), "no prompt is open")
  local buf = vim.api.nvim_win_get_buf(win)
  local lines = {}
  for _, mark in ipairs(vim.api.nvim_buf_get_extmarks(buf, -1, 0, -1, { details = true })) do
    vim.list_extend(lines, mark[4].virt_lines or {})
  end
  return lines
end

---@return string[] each line under the input
function M.below()
  return vim.tbl_map(function(chunks)
    return table.concat(vim.tbl_map(function(chunk)
      return chunk[1]
    end, chunks))
  end, M.below_chunks())
end

---@param text string
---@param ms? integer
function M.below_reads(text, ms)
  assert(
    vim.wait(ms or 2000, function()
      return M.window() and table.concat(M.below(), " ") == text
    end, 5),
    ("the lines under the input never read %q; they read %q"):format(text, table.concat(M.below(), " "))
  )
end

local function reads(text)
  local footer = M.footer()
  local start = footer:match("^(.*)…$")
  return footer == text or (start ~= nil and vim.startswith(text, start))
end

---@param text string
---@param ms? integer
function M.footer_reads(text, ms)
  assert(
    vim.wait(ms or 2000, function()
      return M.window() and reads(text)
    end, 5),
    ("the footer never read %q; it reads %q"):format(text, M.window() and M.footer() or "(closed)")
  )
end

return M
