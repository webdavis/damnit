local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local function with_peek(feature, show)
  local app, fake = helper.new()
  fake.reply("show", fixtures.answer(show or "show.json"))
  require("damnit.peek").new(app, function() end)
  local list = require("damnit.list").new(app)
  require("damnit.command").define(app, { list, require("damnit." .. feature).new(app, list.show) })
  return fake
end

local function opened_agenda(show)
  local fake = with_peek("agenda", show)
  fake.reply("agenda", fixtures.answer("agenda-next-week.json"))
  fake.reply("list", fixtures.answer("list-overdue.json"))
  vim.cmd("Dam agenda")
  buffer.wait_for(function()
    return #fake.of("agenda") > 0 and #buffer.lines() > 1
  end)
  return fake
end

local function peeked_at(fake, line)
  vim.api.nvim_win_set_cursor(0, { vim.fn.index(buffer.lines(), line) + 1, 0 })
  buffer.feed("K")
  buffer.wait_for(function()
    return #fake.of("show") > 0
  end)
  return fake.of("show")[1].argv[3]
end

local function card_lines()
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_config(win).relative ~= "" then
      return vim.api.nvim_buf_get_lines(vim.api.nvim_win_get_buf(win), 0, -1, false)
    end
  end
end

return {
  ["K on an agenda task row peeks at the task"] = function()
    local fake = opened_agenda()
    assert(peeked_at(fake, "[ ] pay rent") == "<id-3>")
  end,

  ["K on an agenda event row shows the event's card"] = function()
    local fake = opened_agenda("show-event.json")
    assert(peeked_at(fake, "09:00–09:30  standup") == "<id-2>")
    buffer.wait_for(function()
      return card_lines() ~= nil
    end)
    assert(card_lines()[1] == "start     2026-09-25T09:00:00+00:00", vim.inspect(card_lines()))
  end,

  ["K on a plan row peeks at the task"] = function()
    local fake = with_peek("plan")
    fake.reply("list", function(_, opts)
      local day = vim.json.decode(opts.stdin)["focus.day"]
      local answer = fixtures.decoded(day == "today" and "plan-today.json" or "plan-empty.json")
      for n, object in ipairs(answer.objects) do
        object.oid = ("%s-%d"):format(day, n)
      end
      return { code = 0, stdout = vim.json.encode(answer), stderr = "" }
    end)
    vim.cmd("Dam plan")
    buffer.wait_for(function()
      return #buffer.lines() > 8
    end)
    assert(peeked_at(fake, "[ ] pay rent") == "today-1")
  end,
}
