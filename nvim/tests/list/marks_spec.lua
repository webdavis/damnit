local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local list = require("helpers.list")

local function marked_lines()
  local lines = {}
  for _, mark in ipairs(vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, { details = true })) do
    if mark[4].line_hl_group == "DamMarked" then
      table.insert(lines, mark[2] + 1)
    end
  end
  return lines
end

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["m marks the row with DamMarked and the winbar counts it"] = function()
    list.opened()
    list.on("[ ] p2 buy oat milk")
    buffer.feed("m")
    same(marked_lines(), { 2 })
    assert(vim.wo.winbar:find("1 marked", 1, true), vim.wo.winbar)
  end,

  ["m on a marked row unmarks it, and [count]m marks that many rows"] = function()
    list.opened()
    list.on("[ ] call mom")
    buffer.feed("2m")
    same(marked_lines(), { 1, 2 })
    buffer.feed("m")
    same(marked_lines(), { 2 })
  end,

  ["m in visual mode marks the selected rows; M unmarks all"] = function()
    list.opened()
    list.on("[ ] call mom")
    buffer.feed("Vjjm")
    same(marked_lines(), { 1, 2, 3 })
    assert(vim.wo.winbar:find("3 marked", 1, true), vim.wo.winbar)
    buffer.feed("M")
    same(marked_lines(), {})
    assert(not vim.wo.winbar:find("marked", 1, true), vim.wo.winbar)
  end,

  ["marks are kept by oid across a redraw and dropped for oids no longer listed"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("3m")
    fake.reply("list", fixtures.answer("list-filter-today.json"))
    buffer.feed("r")
    buffer.wait_for(function()
      return #buffer.lines() == 1
    end)
    same(marked_lines(), { 1 })
    assert(vim.wo.winbar:find("1 marked", 1, true), vim.wo.winbar)
  end,
}
