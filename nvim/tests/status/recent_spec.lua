local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local sections = require("damnit.status.sections")
local status = require("helpers.status")

local function with_recent(commits)
  return vim.tbl_extend("force", fixtures.decoded("status-empty.json"), { recent = commits })
end

local function trimmed(lines)
  return vim.tbl_map(function(line)
    return (line:gsub("%s+$", ""))
  end, lines)
end

return {
  ["Recent lists the newest commits with their short hash, message and change count"] = function()
    local lines, marks = sections.render(with_recent(fixtures.decoded("log.json").commits), 80)
    assert(
      vim.deep_equal(trimmed(lines), {
        "nothing changed, staged, waiting or unpushed",
        "",
        "Recent (3)",
        "  <id-1>  call back",
        "  <id-2>  plan the errand",
        "  <id-3>  seed the week",
      }),
      vim.inspect(lines)
    )
    local counts = vim.tbl_map(
      function(mark)
        return mark.virt[#mark.virt][1]
      end,
      vim.tbl_filter(function(mark)
        return mark.virt ~= nil
      end, marks)
    )
    assert(vim.deep_equal(counts, { "1 change ", "1 change ", "4 changes" }), vim.inspect(counts))
  end,

  ["Recent shows no more than three commits"] = function()
    local commits = fixtures.decoded("log.json").commits
    table.insert(commits, commits[1])
    local lines = sections.render(with_recent(commits), 80)
    assert(lines[3] == "Recent (3)" and #lines == 6, vim.inspect(lines))
  end,

  ["the status window reads dam log and draws Recent last"] = function()
    local _, fake = status.opened()
    buffer.wait_for(function()
      return buffer.lines()[#buffer.lines() - 3] == "Recent (3)"
    end)
    local argv = fake.of("log")[1].argv
    assert(vim.deep_equal(argv, { "dam", "log", "-o", "json" }), vim.inspect(argv))
  end,

  ["a refused dam log leaves Recent out and says nothing"] = function()
    local app, fake = status.defined()
    fake.reply("log", fixtures.refusal("error-refused.json", 4))
    local notes = buffer.capture_notes()
    status.open(fake)
    vim.wait(30)
    assert(not vim.tbl_contains(buffer.lines(), "Recent (3)"), vim.inspect(buffer.lines()))
    assert(#notes == 0 and #app.say:log() == 0, vim.inspect(notes))
  end,
}
