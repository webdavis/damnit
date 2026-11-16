local Keys = {}
Keys.__index = Keys

local M = {}

---@param screen damnit.Screen
---@return damnit.Keys
function M.new(screen)
  return setmetatable({ screen = screen }, Keys)
end

---@param specs damnit.KeySpec[]
---@return string[] each group's heading, then its keys indented under it
function M.help_lines(specs)
  local order, grouped = {}, {}
  for _, spec in ipairs(specs) do
    local group = spec.group or "every window"
    if not grouped[group] then
      grouped[group] = {}
      table.insert(order, group)
    end
    table.insert(grouped[group], "  " .. spec.lhs .. "  " .. spec.desc)
  end
  local lines = {}
  for _, group in ipairs(order) do
    table.insert(lines, group)
    vim.list_extend(lines, grouped[group])
  end
  return lines
end

function Keys:help(specs)
  local lines = M.help_lines(specs)
  local buf = self.screen:scratch("dam://keys", "damkeys")
  local headings = {}
  for n, line in ipairs(lines) do
    if not vim.startswith(line, " ") then
      table.insert(headings, { line = n - 1, col = 0, end_col = #line, group = "DamHeader" })
    end
  end
  self.screen:draw(buf, lines, headings)
  local width = math.max(unpack(vim.tbl_map(vim.fn.strdisplaywidth, lines)))
  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    row = 1,
    col = math.max(vim.o.columns - width - 4, 0),
    width = width,
    height = #lines,
    style = "minimal",
    border = "rounded",
    title = " keys ",
  })
  local function close()
    if vim.api.nvim_win_is_valid(win) then
      vim.api.nvim_win_close(win, true)
    end
  end
  for _, lhs in ipairs({ "q", "<Esc>" }) do
    vim.keymap.set("n", lhs, close, { buffer = buf, nowait = true })
  end
  vim.api.nvim_create_autocmd("WinLeave", { buffer = buf, once = true, callback = close })
end

---@param buf integer
---@param specs damnit.KeySpec[]
function Keys:bind(buf, specs)
  local all = vim.list_slice(specs)
  table.insert(all, {
    lhs = "g?",
    desc = "list these keys",
    group = "every window",
    run = function()
      self:help(all)
    end,
  })
  for _, spec in ipairs(all) do
    vim.keymap.set(spec.mode or "n", spec.lhs, spec.run, { buffer = buf, desc = spec.desc, nowait = true })
  end
end

return M
