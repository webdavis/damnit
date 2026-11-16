local buffer = require("helpers.buffer")
local repeating = require("damnit.kernel.repeat")

local function mapped(log)
  vim.api.nvim_buf_set_lines(0, 0, -1, false, { "one", "two", "three" })
  vim.keymap.set("n", "x", function()
    local count = vim.v.count
    repeating.start(function()
      table.insert(log, { "first", count })
    end, function(again)
      table.insert(log, { "again", again })
    end)
  end, { buffer = true })
end

return {
  ["the key runs once through operatorfunc as a v:lua string"] = function()
    local log = {}
    mapped(log)
    buffer.feed("3x")
    assert(vim.deep_equal(log, { { "first", 3 } }), vim.inspect(log))
    assert(vim.o.operatorfunc:find("^v:lua%.") ~= nil, vim.o.operatorfunc)
  end,

  [". runs it again, with the count typed before the dot"] = function()
    local log = {}
    mapped(log)
    buffer.feed("x.j2.")
    assert(vim.deep_equal(log, { { "first", 0 }, { "again", 0 }, { "again", 2 } }), vim.inspect(log))
  end,

  ["a new key replaces what . repeats"] = function()
    local log = {}
    mapped(log)
    buffer.feed("xx.")
    assert(vim.deep_equal(log, { { "first", 0 }, { "first", 0 }, { "again", 0 } }), vim.inspect(log))
  end,
}
