local fixtures = require("helpers.fixtures")
local lines_of = require("damnit.kernel.confirm_lines")

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

local function groups(marks)
  return vim.tbl_map(function(mark)
    return { mark.line, mark.group }
  end, marks)
end

return {
  ["each change of a preview is a row: op, subject and each field with the value it would have"] = function()
    local lines = lines_of.changes(fixtures.decoded("done-preview.json"))
    same(lines, { "update  call mom  done → yes", "update  pay rent  done → yes", "", "2 of 2 would change" })
  end,

  ["every field dam names shows dam's value for it"] = function()
    local lines = lines_of.changes(fixtures.decoded("edit-preview-fields.json"))
    same(lines[1], "update  pay rent  labels → errand, deadline → 2026-12-04, estimate → 45")
  end,

  ["a row wider than the float is cut to fit"] = function()
    local columns = vim.o.columns
    vim.o.columns = 40
    local lines = lines_of.changes(fixtures.decoded("edit-preview-fields.json"))
    vim.o.columns = columns
    same(lines[1], "update  pay rent  labels → errand…")
  end,

  ["the count line is DamRemoved when the preview removes"] = function()
    local lines, marks = lines_of.changes(fixtures.decoded("rm-preview.json"))
    same(lines, { "delete  call mom", "delete  pay rent", "", "2 of 2 would change" })
    same(groups(marks), { { 0, "DamRemoved" }, { 1, "DamRemoved" }, { 3, "DamRemoved" } })
  end,

  ["a preview that changes nothing says so"] = function()
    local lines, marks = lines_of.changes({ preview = true, count = 2, oids = {}, changes = {} })
    same(lines, { "0 of 2 would change" })
    same(groups(marks), { { 0, "DamMeta" } })
  end,

  ["a push preview lists what each remote would be sent and what it would hold"] = function()
    local answer = fixtures.decoded("push-preview.json")
    answer.remotes[1].held = { { oid = ("9"):rep(40), why = "its path is outside the remote" } }
    local lines = lines_of.push(answer)
    same(lines, {
      ("create  %-38s  fake"):format("pay rent"),
      "held    9999999 its path is outside the remote  fake",
      "",
      "1 would be sent, 1 held",
    })
  end,
}
