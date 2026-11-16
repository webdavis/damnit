local targets = require("damnit.list.targets")

local OIDS = { [1] = "a", [2] = "b", [4] = "d", [5] = "e" }

local function pick(fields)
  return vim.tbl_extend("force", { row = 1, count = 0, marked = {}, lines = 5 }, fields)
end

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["the cursor row alone with no selection, mark or count"] = function()
    same(targets.oids(OIDS, pick({ row = 2 })), { "b" })
  end,

  ["[count] rows from the cursor, the header row left out"] = function()
    same(targets.oids(OIDS, pick({ row = 2, count = 3 })), { "b", "d" })
  end,

  ["a count past the last row stops at the last row"] = function()
    same(targets.oids(OIDS, pick({ row = 4, count = 9 })), { "d", "e" })
  end,

  ["marked rows in screen order win over the count"] = function()
    same(targets.oids(OIDS, pick({ row = 1, count = 2, marked = { e = true, a = true } })), { "a", "e" })
  end,

  ["the visual selection wins over the marks"] = function()
    same(targets.oids(OIDS, pick({ range = { 4, 2 }, marked = { a = true } })), { "b", "d" })
  end,

  ["a header row under the cursor gives no target"] = function()
    same(targets.oids(OIDS, pick({ row = 3 })), {})
  end,
}
