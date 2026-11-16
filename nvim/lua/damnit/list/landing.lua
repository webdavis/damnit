local Landing = {}

---@param list table
function Landing:drawn(list)
  local waiting = list.waiting
  list.waiting = {}
  for _, done in ipairs(waiting) do
    done(list.oids)
  end
end

---@param buf integer
---@return { cwd: string, view: string|damnit.ViewSpec|nil, oid: string|nil }|nil
function Landing:context(buf)
  local list = self.shown[buf]
  if list then
    local row = vim.fn.bufnr() == buf and vim.api.nvim_win_get_cursor(0)[1] or nil
    return { cwd = list.call.cwd, view = list.view, oid = row and list.oids[row] }
  end
end

---@param buf integer
---@param oid string|nil
---@param line integer where the cursor goes when the oid left the view
function Landing:keep_cursor(buf, oid, line)
  table.insert(self.shown[buf].waiting, function(oids)
    local row = math.min(line, vim.api.nvim_buf_line_count(buf))
    for at, listed in pairs(oids) do
      if oid and listed == oid then
        row = at
      end
    end
    for _, win in ipairs(vim.fn.win_findbuf(buf)) do
      vim.api.nvim_win_set_cursor(win, { row, 0 })
    end
  end)
end

---@param buf integer
---@param oid string
---@param done fun(found: boolean)
function Landing:land_on(buf, oid, done)
  table.insert(self.shown[buf].waiting, function(oids)
    local row = vim.fn.index(oids, oid) + 1
    for _, win in ipairs(row > 0 and vim.fn.win_findbuf(buf) or {}) do
      vim.api.nvim_win_set_cursor(win, { row, 0 })
    end
    done(row > 0)
  end)
end

return Landing
