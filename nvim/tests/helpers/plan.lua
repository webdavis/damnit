local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local M = {}

local EXIT_REFUSED = 4

M.ANSWERS = { none = "plan-unplanned.json", today = "plan-today.json", ["today+1"] = "plan-today+1.json" }

---@param full? boolean dam refuses every focus move: the day is full
---@return damnit.App app, table fake
function M.opened(full)
  local app, fake = helper.new()
  local list = require("damnit.list").new(app)
  require("damnit.command").define(app, { list, require("damnit.plan").new(app, list.show) })
  fake.reply("list", function(_, opts)
    local day = vim.json.decode(opts.stdin)["focus.day"]
    local answer = fixtures.decoded(M.ANSWERS[day] or "plan-empty.json")
    for n, object in ipairs(answer.objects) do
      object.oid = ("%s-%d"):format(day, n)
    end
    return { code = 0, stdout = vim.json.encode(answer), stderr = "" }
  end)
  fake.reply(
    "edit",
    full and fixtures.refusal("error-focus-full.json", EXIT_REFUSED) or fixtures.answer("edit-focus.json")
  )
  vim.cmd("Dam plan")
  buffer.wait_for(function()
    return #buffer.lines() > 8
  end)
  return app, fake
end

---@param text string
function M.on(text)
  local row = vim.fn.index(buffer.lines(), text) + 1
  assert(row > 0, ("no line %q in %s"):format(text, vim.inspect(buffer.lines())))
  vim.api.nvim_win_set_cursor(0, { row, 0 })
end

---@param fake table
---@param count integer
---@param refused? boolean dam refused the edits, so the plan is not read again
---@return string[] argv of the last edit
function M.edited(fake, count, refused)
  local sections = #require("damnit.plan.anchors").sections()
  buffer.wait_for(function()
    return #fake.of("edit") == count and (refused or #fake.of("list") == sections * (count + 1))
  end)
  local calls = fake.of("edit")
  return calls[#calls].argv
end

return M
