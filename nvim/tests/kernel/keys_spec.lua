local buffer = require("helpers.buffer")
local helper = require("helpers.app")

return {
  ["g? lists every bound key with its description"] = function()
    local app = helper.new()
    local buf = app.screen:scratch("dam://keys-spec", "damspec")
    app.screen:show(buf)
    local ran = false
    app.keys:bind(buf, {
      { lhs = "x", desc = "mark the task done", run = function() end },
      {
        lhs = "r",
        desc = "refresh",
        run = function()
          ran = true
        end,
      },
    })
    buffer.feed("r")
    assert(ran, "r did not run its spec")
    buffer.feed("g?")
    local help = table.concat(buffer.lines(), "\n")
    for _, line in ipairs({ "x  mark the task done", "r  refresh", "g?  list these keys" }) do
      assert(help:find(line, 1, true), help)
    end
  end,
}
