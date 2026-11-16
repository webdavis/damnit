local screen_of = require("damnit.kernel.screen")

local function drawn(marks)
  local screen = screen_of.new()
  local buf = screen:scratch("dam://spec", "damspec")
  vim.api.nvim_win_set_buf(0, buf)
  screen:draw(buf, { "first row", "second row" }, marks)
  return buf, screen
end

local function details(buf, screen)
  return vim.tbl_map(function(mark)
    return mark[4]
  end, vim.api.nvim_buf_get_extmarks(buf, screen.namespace, 0, -1, { details = true }))
end

return {
  ["a mark carries right-aligned virtual text at the end of its line"] = function()
    local buf, screen = drawn({ { line = 1, virt = { { "2026-09-25", "DamDue" } }, align = "eol_right_align" } })
    local mark = details(buf, screen)[1]
    assert(mark.virt_text_pos == "eol_right_align", vim.inspect(mark))
    assert(vim.deep_equal(mark.virt_text, { { "2026-09-25", "DamDue" } }), vim.inspect(mark))
  end,

  ["a mark carries a sign, a line highlight and virtual lines"] = function()
    local buf, screen = drawn({
      { line = 0, sign = "!", sign_group = "DamOverdue" },
      { line = 1, line_group = "DamMarked" },
      { line = 1, virt_lines = { { { "priority  4 -> 2", "DamMeta" } } } },
    })
    local marks = details(buf, screen)
    assert(vim.trim(marks[1].sign_text) == "!" and marks[1].sign_hl_group == "DamOverdue", vim.inspect(marks))
    assert(marks[2].line_hl_group == "DamMarked", vim.inspect(marks))
    assert(vim.deep_equal(marks[3].virt_lines, { { { "priority  4 -> 2", "DamMeta" } } }), vim.inspect(marks))
  end,

  ["a highlight mark still covers its columns"] = function()
    local buf, screen = drawn({ { line = 0, col = 0, end_col = 5, group = "DamDone" } })
    local mark = details(buf, screen)[1]
    assert(mark.hl_group == "DamDone" and mark.end_col == 5, vim.inspect(mark))
  end,

  ["drawing again keeps one namespace and drops the old marks"] = function()
    local buf, screen = drawn({ { line = 0, sign = "!", sign_group = "DamOverdue" } })
    screen:draw(buf, { "only row" }, { { line = 0, sign = "~", sign_group = "DamBlocked" } })
    local marks = details(buf, screen)
    assert(#marks == 1 and vim.trim(marks[1].sign_text) == "~", vim.inspect(marks))
  end,

  ["a window look sets its options on every window showing the buffer and again when shown"] = function()
    local screen = screen_of.new()
    local buf = screen:scratch("dam://spec", "damspec")
    vim.api.nvim_win_set_buf(0, buf)
    screen:window(buf, { winbar = screen_of.bar({ { " today", "DamHeader" }, { " 100%" } }), signcolumn = "auto:2" })
    assert(vim.wo.winbar == "%#DamHeader# today%* 100%%", vim.wo.winbar)
    assert(vim.wo.signcolumn == "auto:2", vim.wo.signcolumn)
    vim.cmd.split()
    vim.cmd.enew()
    assert(vim.wo.winbar == "", vim.wo.winbar)
    vim.api.nvim_win_set_buf(0, buf)
    assert(vim.wo.winbar == "%#DamHeader# today%* 100%%", vim.wo.winbar)
  end,

  ["a winbar cuts at its cut chunk, so what comes before it stays"] = function()
    local bar = screen_of.bar({ { " today", "DamHeader" }, { " · app/" }, { "", cut = true }, { " · 6 tasks" } })
    assert(bar == "%#DamHeader# today%* · app/%< · 6 tasks", bar)
  end,

  ["a window look can turn wrap off and gives the text width of the narrowest window"] = function()
    local screen = screen_of.new()
    local buf = screen:scratch("dam://spec", "damspec")
    vim.api.nvim_win_set_buf(0, buf)
    screen:window(buf, { wrap = false })
    assert(vim.wo.wrap == false)
    assert(screen:text_width(buf) == vim.api.nvim_win_get_width(0), screen:text_width(buf))
    vim.cmd("vertical 10split")
    assert(screen:text_width(buf) == 10, screen:text_width(buf))
  end,
}
