local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local wait_for = require("helpers.buffer").wait_for

local EXIT_USAGE = 2

local function finished(got)
  wait_for(function()
    return got.done
  end)
  return got
end

return {
  ["a call runs dam with -o json in the folder it was given"] = function()
    local app, fake = helper.new()
    fake.reply("list", fixtures.answer("list.json"))
    local got = finished(helper.call(app, { args = { "list" }, cwd = "/projects/garden" }))
    local list = fake.of("list")[1]
    assert(vim.deep_equal(list.argv, { "dam", "list", "-o", "json" }), vim.inspect(list.argv))
    assert(list.cwd == "/projects/garden", list.cwd)
    assert(got.answer.objects[2].subject == "buy oat milk", vim.inspect(got.answer))
  end,

  ["stdin is passed for --request -"] = function()
    local app, fake = helper.new()
    fake.reply("list", fixtures.answer("list.json"))
    finished(helper.call(app, { args = { "list", "--request", "-" }, cwd = "/", stdin = '{"overdue":true}' }))
    assert(fake.of("list")[1].stdin == '{"overdue":true}')
  end,

  ["an exit with an error document becomes that error with its suggested command"] = function()
    local app, fake = helper.new()
    fake.reply("list", fixtures.refusal("error-usage-selection.json", EXIT_USAGE))
    local got = finished(helper.call(app, { args = { "list", "--priority", "urgent" }, cwd = "/" }))
    assert(got.err.kind == "usage", vim.inspect(got.err))
    assert(got.err.selection.command == "dam list --priority 1", vim.inspect(got.err))
  end,

  ["output that is not JSON becomes a plugin error carrying stderr"] = function()
    local app, fake = helper.new()
    fake.reply("list", { code = 0, stdout = "objects: []", stderr = "warning: config is world readable" })
    local got = finished(helper.call(app, { args = { "list" }, cwd = "/" }))
    assert(got.err.kind == "plugin", vim.inspect(got.err))
    assert(got.err.message:find("config is world readable", 1, true), got.err.message)
  end,

  ["a dam outside the supported range refuses every call and says which version"] = function()
    local app, fake = helper.new()
    fake.reply("--version", { code = 0, stdout = "dam 9.1.0\n", stderr = "" })
    fake.reply("list", fixtures.answer("list.json"))
    local first = finished(helper.call(app, { args = { "list" }, cwd = "/" }))
    local second = finished(helper.call(app, { args = { "list" }, cwd = "/" }))
    for _, got in ipairs({ first, second }) do
      assert(got.err.kind == "plugin" and got.err.message:find("dam 9.1.0 is outside", 1, true), vim.inspect(got.err))
    end
    assert(#fake.of("list") == 0, "dam list ran under an unsupported dam")
  end,

  ["a missing dam binary names the option to set"] = function()
    local app, fake = helper.new({ dam = "/opt/nowhere/dam" })
    fake.reply("--version", { missing = true })
    local got = finished(helper.call(app, { args = { "list" }, cwd = "/" }))
    assert(got.err.kind == "plugin", vim.inspect(got.err))
    assert(got.err.message:find("/opt/nowhere/dam", 1, true), got.err.message)
    assert(got.err.message:find("dam = ", 1, true), got.err.message)
  end,

  ["the version is asked once for many calls"] = function()
    local app, fake = helper.new()
    fake.reply("list", fixtures.answer("list.json"))
    local calls = {}
    for n = 1, 3 do
      calls[n] = helper.call(app, { args = { "list" }, cwd = "/", key = "lane " .. n })
    end
    for _, got in ipairs(calls) do
      finished(got)
    end
    finished(helper.call(app, { args = { "list" }, cwd = "/" }))
    assert(#fake.of("--version") == 1, ("asked %d times"):format(#fake.of("--version")))
    assert(#fake.of("list") == 4)
  end,

  ["a cancelled call never calls back"] = function()
    local app, fake = helper.new()
    fake.reply("list", vim.tbl_extend("force", fixtures.answer("list.json"), { hold = true }))
    local got = helper.call(app, { args = { "list" }, cwd = "/" })
    wait_for(function()
      return #fake.of("list") == 1
    end)
    got.handle.cancel()
    fake.release(fake.of("list")[1])
    vim.wait(50)
    assert(fake.of("list")[1].killed, "dam was not stopped")
    assert(not got.done, "the cancelled call called back")
  end,

  ["a timeout kills dam and reports it"] = function()
    local app, fake = helper.new({ timeout_ms = 20 })
    fake.reply("pull", { code = 0, stdout = "{}", stderr = "", hold = true })
    local got = finished(helper.call(app, { args = { "pull" }, cwd = "/" }))
    assert(fake.of("pull")[1].killed, "dam was left running")
    assert(
      got.err.kind == "plugin" and got.err.message:find("dam pull took longer than 20 ms", 1, true),
      vim.inspect(got.err)
    )
  end,
}
