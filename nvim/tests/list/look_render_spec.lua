local fixtures = require("helpers.fixtures")
local look = require("damnit.list.look")
local tree = require("damnit.list.tree")

local SIGNS = require("damnit.kernel.options").defaults().signs

local function aligned_texts(marks)
  local texts = {}
  for _, mark in ipairs(marks) do
    if mark.align then
      texts[mark.line + 1] = table.concat(vim.tbl_map(function(chunk)
        return chunk[1]
      end, mark.virt))
    end
  end
  return texts
end

return {
  ["a row too long to hold its due does not drop the other rows' columns"] = function()
    local objects = fixtures.decoded("list-tree.json").objects
    objects[1].subject = ("a long subject "):rep(5)
    local _, marks = look.render(tree.arranged(objects), SIGNS, 78)
    local texts = aligned_texts(marks)
    assert(texts[4]:find("context=errand", 1, true), vim.inspect(texts))
  end,

  ["when no row can hold its due beside its text only the due is kept"] = function()
    local objects = fixtures.decoded("list-tree.json").objects
    for _, object in ipairs(objects) do
      object.subject, object.depends = object.subject .. ("!"):rep(40), {}
    end
    local _, marks = look.render(tree.arranged(objects), SIGNS, 52)
    local texts = aligned_texts(marks)
    assert(texts[1] == nil and texts[4] == "… 2026-09-25", vim.inspect(texts))
  end,
}
