local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local wait_for = require("helpers.buffer").wait_for

local function held(name)
  return vim.tbl_extend("force", fixtures.answer(name), { hold = true })
end

return {
  ["a newer read in a lane cancels the older one"] = function()
    local app, fake = helper.new()
    fake.reply("list", held("list.json"))
    local older = helper.call(app, { args = { "list" }, cwd = "/", key = "view" })
    wait_for(function()
      return #fake.of("list") == 1
    end)
    local newer = helper.call(app, { args = { "list", "--overdue" }, cwd = "/", key = "view" })
    wait_for(function()
      return #fake.of("list") == 2
    end)
    fake.release(fake.of("list")[2])
    wait_for(function()
      return newer.done
    end)
    assert(fake.of("list")[1].killed, "the older read was left running")
    assert(not older.done, "the older read called back")
  end,

  ["writes in a lane run in the order sent"] = function()
    local app, fake = helper.new()
    fake.reply("done", { code = 0, stdout = "{}", stderr = "", hold = true })
    local first = helper.call(app, { args = { "done", "aaa" }, cwd = "/", key = "edits" }, "write")
    local second = helper.call(app, { args = { "done", "bbb" }, cwd = "/", key = "edits" }, "write")
    wait_for(function()
      return #fake.of("done") == 1
    end)
    vim.wait(30)
    assert(#fake.of("done") == 1, "the second write started before the first finished")
    fake.release(fake.of("done")[1])
    wait_for(function()
      return #fake.of("done") == 2
    end)
    fake.release(fake.of("done")[2])
    wait_for(function()
      return second.done
    end)
    assert(first.done and not first.err and not second.err)
    assert(fake.of("done")[1].argv[3] == "aaa" and fake.of("done")[2].argv[3] == "bbb")
  end,
}
