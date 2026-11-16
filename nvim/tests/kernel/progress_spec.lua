local fixtures = require("helpers.fixtures")
local progress = require("damnit.kernel.progress")

local function lines_of(name)
  return vim.split(vim.trim(fixtures.read(name)), "\n", { plain = true })
end

return {
  ["a record line becomes a progress message with the step as its text and the remote as its title"] = function()
    local text, opts = progress.message(assert(progress.record(lines_of("progress-pull.txt")[1])))
    assert(text == "pull", text)
    assert(
      vim.deep_equal(opts, { kind = "progress", status = "running", title = "fake", source = "dam" }),
      vim.inspect(opts)
    )
  end,

  ["a finished record ends its message as success"] = function()
    local _, opts = progress.message(assert(progress.record(lines_of("progress-pull.txt")[2])))
    assert(opts.status == "success", vim.inspect(opts))
  end,

  ["a failed record ends its message as failed"] = function()
    local _, opts = progress.message(assert(progress.record(lines_of("progress-push-failed.txt")[2])))
    assert(opts.status == "failed", vim.inspect(opts))
  end,

  ["a percent is given only when dam gives a total above 0"] = function()
    local push = lines_of("progress-push.txt")
    local _, started = progress.message(assert(progress.record(push[1])))
    local _, finished = progress.message(assert(progress.record(push[2])))
    local _, pull = progress.message(assert(progress.record(lines_of("progress-pull.txt")[1])))
    local _, none = progress.message({ step = "change", stage = "started", done = 0, total = 0, seconds = 0 })
    assert(started.percent == 0 and finished.percent == 100, vim.inspect({ started, finished }))
    assert(pull.percent == nil and none.percent == nil, vim.inspect({ pull, none }))
  end,

  ["lines that are not records are not read as records"] = function()
    local failed = lines_of("progress-push-failed.txt")
    for n = 3, #failed do
      assert(progress.record(failed[n]) == nil, failed[n])
    end
    for _, line in ipairs({ "", "warning: config is world readable", '{"step":"push"}', "[1]" }) do
      assert(progress.record(line) == nil, line)
    end
  end,
}
