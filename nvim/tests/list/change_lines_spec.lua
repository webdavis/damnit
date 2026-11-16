local change_lines = require("damnit.list.change_lines")
local fixtures = require("helpers.fixtures")

local OAT_MILK, CALL_MOM, RENT = ("1"):rep(40), ("2"):rep(40), "<id-1>"

local OBJECTS = {
  [OAT_MILK] = {
    oid = OAT_MILK,
    subject = "buy oat milk",
    labels = { "errand" },
    task = { done = false, due = "2026-09-25" },
  },
  [CALL_MOM] = { oid = CALL_MOM, subject = "call mom", labels = {}, task = { done = false } },
  [RENT] = { oid = RENT, subject = "pay rent", labels = {}, task = { done = false, due = "2026-09-30", estimate = 30 } },
}

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["each target is a row: subject, field, before from the row and after from dam's preview"] = function()
    local lines = change_lines.rows(fixtures.decoded("edit-preview.json"), OBJECTS, "due")
    same(lines, {
      "buy oat milk  due  2026-09-25 → 2026-11-20",
      "call mom      due  none → 2026-11-20",
      "pay rent      due  2026-09-30 → 2026-11-20",
    })
  end,

  ["one target reads as the field, before and after"] = function()
    local answer = fixtures.decoded("mv-preview.json")
    same(change_lines.reading(answer, OBJECTS, "path"), "path  none → work/")
  end,

  ["a field dam's preview does not print reads as changed"] = function()
    local answer = fixtures.decoded("mv-preview.json")
    answer.changes[1].fields = { "estimate" }
    same(change_lines.reading(answer, OBJECTS, "estimate"), "estimate  30 → changed")
  end,

  ["with no field named, each field dam lists is a row"] = function()
    local answer = fixtures.decoded("done-preview.json")
    same(change_lines.rows(answer, OBJECTS), { "call mom  done  no → yes", "pay rent  done  no → yes" })
  end,

  ["a target dam would not change says so"] = function()
    local answer = { preview = true, count = 1, oids = { CALL_MOM }, changes = {} }
    same(change_lines.reading(answer, OBJECTS, "due"), "no change")
    same(change_lines.rows(answer, OBJECTS, "due"), { "call mom  unchanged" })
  end,
}
