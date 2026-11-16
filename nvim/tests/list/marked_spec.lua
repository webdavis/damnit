local marked = require("damnit.list.marked")

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["m marks unmarked rows and unmarks marked ones"] = function()
    same(marked.toggled({ a = true }, { "a", "b" }), { b = true })
  end,

  ["marks are kept by oid and dropped for oids no longer listed"] = function()
    same(marked.kept({ a = true, gone = true }, { [1] = "b", [3] = "a" }), { a = true })
  end,

  ["a marked row has the DamMarked line group"] = function()
    same(marked.marks({ a = true }, { [1] = "b", [3] = "a" }), { { line = 2, line_group = "DamMarked" } })
  end,

  ["the count of marked rows"] = function()
    assert(marked.count({ a = true, c = true }) == 2)
    assert(marked.count({}) == 0)
  end,
}
