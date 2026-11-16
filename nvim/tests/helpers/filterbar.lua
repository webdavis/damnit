local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local prompt = require("helpers.prompt")

local M = {}

local EXIT_USAGE = 2

M.WEEK = { week = { due = "..today+7", labels = "bug" } }

local function listing(argv)
  if vim.tbl_contains(argv, "--overdue") then
    return fixtures.answer("list-filter-today.json")
  end
  return fixtures.answer("list.json")
end

---@param options? table
---@param command? string
---@return damnit.App app, table fake
function M.opened(options, command)
  local app, fake = helper.new(vim.tbl_extend("force", { views = M.WEEK }, options or {}))
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", listing)
  fake.reply("filter", function(argv)
    return fixtures.answer(argv[3] == "create" and "filter-create.json" or "filter-list.json")
  end)
  for _, verb in ipairs({ "category", "remote", "label", "path" }) do
    fake.reply(verb, fixtures.refusal("error-unknown-subcommand.json", EXIT_USAGE))
  end
  vim.cmd(command or "Dam list week")
  buffer.wait_for(function()
    return #fake.of("list") > 0 and #buffer.lines() > 1
  end)
  return app, fake
end

---@param fake table
---@return string[][] each dam list call's words after dam, without -o json
function M.lists(fake)
  return vim.tbl_map(function(call)
    return vim.list_slice(call.argv, 2, #call.argv - 2)
  end, fake.of("list"))
end

---@param fake table
---@param count integer
function M.wait_lists(fake, count)
  buffer.wait_for(function()
    return #fake.of("list") >= count
  end)
  vim.wait(20)
end

---@param keys? string
function M.bar(keys)
  prompt.type("f" .. (keys or ""))
  assert(prompt.window(), "no filter bar opened")
end

return M
