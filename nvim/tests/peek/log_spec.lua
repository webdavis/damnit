local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local log = require("helpers.log")

local MILK = ("1"):rep(40)

local function log_with_peek()
  local app, fake = log.defined()
  fake.reply("show", fixtures.answer("show.json"))
  require("damnit.peek").new(app, function() end)
  log.open(fake)
  return fake
end

local function on_commit(message)
  for n, line in ipairs(buffer.lines()) do
    if line:find(message, 1, true) then
      return vim.api.nvim_win_set_cursor(0, { n, 0 })
    end
  end
  error("no commit row " .. message)
end

return {
  ["K on a log row peeks at the first task the commit changed"] = function()
    local fake = log_with_peek()
    on_commit("seed the week")
    buffer.feed("K")
    buffer.wait_for(function()
      return #fake.of("show") > 0
    end)
    assert(fake.of("show")[1].argv[3] == MILK, vim.inspect(fake.of("show")[1].argv))
  end,

  ["K is listed by g? under the log's keys"] = function()
    log_with_peek()
    buffer.feed("g?")
    local lines = buffer.lines()
    local heading = vim.fn.index(lines, "log")
    assert(
      heading >= 0 and vim.fn.index(lines, "  K  peek at the task, or enter the peek") > heading,
      vim.inspect(lines)
    )
  end,
}
