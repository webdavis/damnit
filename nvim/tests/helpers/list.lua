local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local M = {}

M.CALL_MOM, M.OAT_MILK, M.BAG = ("2"):rep(40), ("1"):rep(40), "<id-1>"

local EXIT_USAGE = 2

local ANSWERS = {
  done = { "done.json", "done-preview.json", "done-preview.json" },
  edit = { "edit.json", "edit-many.json", "edit-preview.json" },
  mv = { "mv.json", "edit-many.json", "mv-preview.json" },
  rm = { "rm.json", "rm.json", "rm-preview.json" },
  restore = { "restore.json", "restore.json", "restore-preview.json" },
  add = { "status-add.json", "status-add.json", "add-preview.json" },
}

local function answering(names)
  return function(argv)
    if vim.tbl_contains(argv, "--preview") then
      return fixtures.answer(names[3])
    end
    return fixtures.answer(vim.tbl_contains(argv, "-") and names[2] or names[1])
  end
end

---@param options? table
---@return damnit.App app, table fake
function M.opened(options)
  local app, fake = helper.new(options)
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", fixtures.answer("list.json"))
  for verb, names in pairs(ANSWERS) do
    fake.reply(verb, answering(names))
  end
  for _, verb in ipairs({ "filter", "category", "remote", "label", "path" }) do
    fake.reply(verb, fixtures.refusal("error-unknown-subcommand.json", EXIT_USAGE))
  end
  vim.cmd("Dam list")
  buffer.wait_for(function()
    return #fake.of("list") > 0 and #buffer.lines() > 1
  end)
  return app, fake
end

---@param text string
function M.on(text)
  local row = vim.fn.index(buffer.lines(), text) + 1
  assert(row > 0, ("no row %q in %s"):format(text, vim.inspect(buffer.lines())))
  vim.api.nvim_win_set_cursor(0, { row, 0 })
end

---@param fake table
---@param verb string
---@param preview boolean
---@return table[] calls, each { argv, stdin }
function M.sent(fake, verb, preview)
  return vim.tbl_map(
    function(call)
      return { argv = vim.list_slice(call.argv, 2), stdin = call.stdin }
    end,
    vim.tbl_filter(function(call)
      return vim.tbl_contains(call.argv, "--preview") == preview
    end, fake.of(verb))
  )
end

---@param fake table
---@param verb string
---@param count integer
---@param preview? boolean
---@return table[] calls
function M.wait_sent(fake, verb, count, preview)
  buffer.wait_for(function()
    return #M.sent(fake, verb, preview == true) >= count
  end)
  vim.wait(20)
  return M.sent(fake, verb, preview == true)
end

---@return integer|nil
function M.confirm_window()
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.bo[vim.api.nvim_win_get_buf(win)].filetype == "damconfirm" then
      return win
    end
  end
end

---@param fake table
---@param verb string
function M.wait_confirm(fake, verb)
  M.wait_sent(fake, verb, 1, true)
  buffer.wait_for(function()
    return M.confirm_window() ~= nil
  end)
end

return M
