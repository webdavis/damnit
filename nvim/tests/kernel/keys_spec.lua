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

  ["g? lists the keys under their group's heading, each group once, in the order first bound"] = function()
    local lines = require("damnit.kernel.keys").help_lines({
      { lhs = "<CR>", desc = "open the task", group = "every window", run = function() end },
      { lhs = "x", desc = "done", group = "task rows", run = function() end },
      { lhs = "r", desc = "read again", group = "every window", run = function() end },
    })
    local want = { "every window", "  <CR>  open the task", "  r  read again", "task rows", "  x  done" }
    assert(vim.deep_equal(lines, want), vim.inspect(lines))
  end,

  ["g? in a list groups every key as the design's key table does"] = function()
    require("helpers.list").opened()
    require("helpers.buffer").feed("g?")
    local help = require("helpers.buffer").lines()
    assert(help[1] == "every window", vim.inspect(help))
    local rows = vim.fn.index(help, "task rows")
    assert(rows > 0 and vim.tbl_contains(help, "  cd  change the due date"), vim.inspect(help))
    assert(vim.fn.index(help, "  <CR>  open the task") < rows and vim.fn.index(help, "  a  new task") > rows)
  end,

  ["a key added everywhere is bound in each window bound after, under every window"] = function()
    local app = helper.new()
    local ran = false
    app.keys:everywhere({
      lhs = "gL",
      desc = "open the log",
      run = function()
        ran = true
      end,
    })
    local buf = app.screen:scratch("dam://keys-everywhere", "damspec")
    app.screen:show(buf)
    app.keys:bind(buf, { { lhs = "x", desc = "done", group = "list", run = function() end } })
    buffer.feed("gL")
    assert(ran, "gL did not run")
    buffer.feed("g?")
    local help = buffer.lines()
    assert(
      vim.deep_equal(vim.list_slice(help, 3), { "every window", "  gL  open the log", "  g?  list these keys" }),
      vim.inspect(help)
    )
  end,
}
