local buffer = require("helpers.buffer")
local confirm = require("damnit.kernel.confirm")
local fixtures = require("helpers.fixtures")
local screen = require("damnit.kernel.screen")

local function floating()
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_config(win).relative ~= "" then
      return win
    end
  end
end

local function opened(said)
  local lines, marks = require("damnit.kernel.confirm_lines").changes(fixtures.decoded("rm-preview.json"))
  confirm.open(screen.new(), {
    title = "dam rm",
    lines = lines,
    marks = marks,
    yes = function()
      table.insert(said, "yes")
    end,
  })
  return assert(floating(), "no float opened")
end

return {
  ["the float shows dam's preview under a title naming the command, and keys in the footer"] = function()
    local win = opened({})
    local config = vim.api.nvim_win_get_config(win)
    assert(vim.trim(config.title[1][1]) == "dam rm", vim.inspect(config.title))
    assert(vim.trim(config.footer[1][1]) == "y yes · n no", vim.inspect(config.footer))
    local buf = vim.api.nvim_win_get_buf(win)
    assert(buffer.lines(buf)[4] == "2 of 2 would change", vim.inspect(buffer.lines(buf)))
    local marks = vim.api.nvim_buf_get_extmarks(buf, -1, { 3, 0 }, { 3, -1 }, { details = true })
    assert(marks[1] and marks[1][4].hl_group == "DamRemoved", vim.inspect(marks))
  end,

  ["y and <CR> do it and close the float"] = function()
    for _, key in ipairs({ "y", "<CR>" }) do
      local said = {}
      opened(said)
      buffer.feed(key)
      assert(vim.deep_equal(said, { "yes" }) and floating() == nil, key)
    end
  end,

  ["n, <Esc> and q keep and close the float"] = function()
    for _, key in ipairs({ "n", "<Esc>", "q" }) do
      local said = {}
      opened(said)
      buffer.feed(key)
      assert(#said == 0 and floating() == nil, key)
    end
  end,
}
