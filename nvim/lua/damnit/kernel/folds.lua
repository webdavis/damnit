local M = {}

---@class damnit.FoldLine
---@field header? boolean
---@field depth? integer
---@field open? integer the children dam counts as not done, shown on a closed parent
---@field key? string what names the line from one draw to the next, so a closed fold stays closed

---@param lines damnit.FoldLine[]
---@return (string|integer)[]
function M.levels(lines)
  local levels, base = {}, 0
  for n, line in ipairs(lines) do
    local next_line = lines[n + 1]
    if line.header then
      base, levels[n] = 1, ">1"
    elseif not line.depth then
      levels[n] = 0
    elseif next_line and next_line.depth and next_line.depth > line.depth then
      levels[n] = ">" .. (base + line.depth + 1)
    else
      levels[n] = base + line.depth
    end
  end
  return levels
end

---@param lnum integer
---@return string|integer
function M.expr(lnum)
  return (vim.b.damnit_folds or {}).levels[lnum] or 0
end

---@return string[][]
function M.text()
  local start = vim.v.foldstart
  local line = (vim.b.damnit_folds or { lines = {} }).lines[start] or {}
  local chunks = { { vim.fn.getline(start), line.header and "DamHeader" or nil } }
  if (line.open or 0) > 0 then
    table.insert(chunks, { ("  +%d open"):format(line.open), "DamMeta" })
  end
  table.insert(chunks, { " " })
  return chunks
end

M.window = {
  foldmethod = "expr",
  foldexpr = "v:lua.require'damnit.kernel.folds'.expr(v:lnum)",
  foldtext = "v:lua.require'damnit.kernel.folds'.text()",
  foldlevel = 99,
}

---@param win integer
---@param lines damnit.FoldLine[]
---@return table<string, boolean>
local function closed_keys(win, lines)
  local closed = {}
  vim.api.nvim_win_call(win, function()
    for n, line in ipairs(lines) do
      if line.key and vim.fn.foldclosed(n) == n then
        closed[line.key] = true
      end
    end
  end)
  return closed
end

---@param win integer
---@param lines damnit.FoldLine[]
---@param closed table<string, boolean>
local function close_again(win, lines, closed)
  vim.api.nvim_win_call(win, function()
    for n = #lines, 1, -1 do
      if closed[lines[n].key] then
        vim.cmd(n .. "foldclose")
      end
    end
  end)
end

---@param buf integer
---@param lines damnit.FoldLine[]
---@param draw fun() draws the buffer's lines and window
function M.redraw(buf, lines, draw)
  local before = (vim.b[buf].damnit_folds or { lines = {} }).lines
  local closed = {}
  for _, win in ipairs(vim.fn.win_findbuf(buf)) do
    closed[win] = closed_keys(win, before)
  end
  vim.b[buf].damnit_folds = { levels = M.levels(lines), lines = lines }
  draw()
  for _, win in ipairs(vim.fn.win_findbuf(buf)) do
    close_again(win, lines, closed[win] or {})
  end
end

---@param step 1|-1
local function jump(step)
  local lines = (vim.b.damnit_folds or { lines = {} }).lines
  local n = vim.api.nvim_win_get_cursor(0)[1] + step
  while lines[n] do
    if lines[n].header then
      return vim.api.nvim_win_set_cursor(0, { n, 0 })
    end
    n = n + step
  end
end

---@return damnit.KeySpec[]
function M.keys()
  return {
    {
      lhs = "<Tab>",
      desc = "open or close the fold under the cursor",
      run = function()
        if vim.fn.foldlevel(".") > 0 then
          vim.cmd("normal! za")
        end
      end,
    },
    {
      lhs = "[[",
      desc = "previous header",
      run = function()
        jump(-1)
      end,
    },
    {
      lhs = "]]",
      desc = "next header",
      run = function()
        jump(1)
      end,
    },
  }
end

return M
