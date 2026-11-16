local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local buffer = require("helpers.buffer")

local EXIT_REFUSED = 4

local function changes()
  local seen = {}
  vim.api.nvim_create_autocmd("User", {
    group = "DamSpec",
    pattern = "DamChanged",
    callback = function(event)
      table.insert(seen, event.data)
    end,
  })
  return seen
end

return {
  ["a successful write fires DamChanged"] = function()
    local app, fake = helper.new()
    fake.reply("done", { code = 0, stdout = "{}", stderr = "" })
    local seen = changes()
    local got = helper.call(app, { args = { "done", "aaa" }, cwd = "/projects/garden" }, "write")
    buffer.wait_for(function()
      return got.done
    end)
    assert(#seen == 1 and seen[1].cwd == "/projects/garden", vim.inspect(seen))
  end,

  ["a failed write fires nothing and shows the error"] = function()
    local app, fake = helper.new()
    fake.reply("show", fixtures.refusal("error-refused.json", EXIT_REFUSED))
    local seen = changes()
    local notes = buffer.capture_notes()
    local got = helper.call(app, { args = { "show", "0000000" }, cwd = "/" }, "write")
    buffer.wait_for(function()
      return got.done
    end)
    assert(#seen == 0, "DamChanged fired for a refusal")
    assert(#notes == 1 and notes[1].text:find("no object matches", 1, true), vim.inspect(notes))
    assert(notes[1].level == vim.log.levels.ERROR)
  end,

  ["a placed write keeps its refusal for :Dam messages and announces nothing"] = function()
    local app, fake = helper.new()
    fake.reply("show", fixtures.refusal("error-refused.json", EXIT_REFUSED))
    local notes = buffer.capture_notes()
    local got = helper.call(app, { args = { "show", "0000000" }, cwd = "/", placed = true }, "write")
    buffer.wait_for(function()
      return got.done
    end)
    vim.wait(20)
    assert(#notes == 0, vim.inspect(notes))
    assert(#app.say:log() == 1 and got.err == app.say:log()[1], vim.inspect(app.say:log()))
  end,
}
