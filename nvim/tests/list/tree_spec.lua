local fixtures = require("helpers.fixtures")
local tree = require("damnit.list.tree")

local function task(oid, parent)
  return { oid = oid, parent = parent }
end

local function shape(objects)
  return vim.tbl_map(function(row)
    return string.rep("  ", row.depth) .. row.object.oid
  end, tree.arranged(objects))
end

local function assert_shape(objects, expected)
  local got = shape(objects)
  assert(vim.deep_equal(got, expected), vim.inspect(got))
end

return {
  ["children listed after an unrelated task are drawn under their own parent"] = function()
    assert_shape(
      { task("offsite"), task("rent"), task("room", "offsite"), task("invites", "offsite") },
      { "offsite", "  room", "  invites", "rent" }
    )
    local subjects = vim.tbl_map(function(row)
      return string.rep("  ", row.depth) .. row.object.subject
    end, tree.arranged(fixtures.decoded("list-tree.json").objects))
    local parent = assert(vim.iter(ipairs(subjects)):find(function(_, subject)
      return subject == "buy oat milk"
    end))
    assert(subjects[parent + 1] == "  pack the bag", vim.inspect(subjects))
  end,

  ["a child is indented under its parent"] = function()
    assert_shape({ task("a"), task("b", "a") }, { "a", "  b" })
  end,

  ["grandchildren sit under their own parent"] = function()
    assert_shape({ task("a"), task("b"), task("c", "a1"), task("a1", "a") }, { "a", "  a1", "    c", "b" })
  end,

  ["siblings keep dam's order"] = function()
    assert_shape({ task("z", "p"), task("p"), task("a", "p") }, { "p", "  z", "  a" })
  end,

  ["a child whose parent is not listed stays a root in its place"] = function()
    assert_shape({ task("a"), task("b", "gone"), task("c") }, { "a", "b", "c" })
  end,

  ["a parent chain that loops does not hang"] = function()
    local got = shape({ task("a", "b"), task("b", "a"), task("c") })
    assert(#got == 3 and got[1] == "c", vim.inspect(got))
  end,
}
