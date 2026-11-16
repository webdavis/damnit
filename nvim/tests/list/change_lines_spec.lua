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

  ["the deadline, estimate and category read as the value dam's preview gives"] = function()
    local answer = fixtures.decoded("edit-preview-fields.json")
    same(change_lines.reading(answer, OBJECTS, "deadline"), "deadline  none → 2026-12-04")
    same(change_lines.reading(answer, OBJECTS, "estimate"), "estimate  30 → 45")
    same(change_lines.reading(answer, OBJECTS, "category"), "category  none → context=errand")
  end,

  ["a field dam's preview clears reads as none"] = function()
    local answer = fixtures.decoded("edit-preview-fields.json")
    answer.changes[1].fields, answer.changes[1].estimate = { "estimate" }, nil
    same(change_lines.reading(answer, OBJECTS, "estimate"), "estimate  30 → none")
  end,

  ["an event's field reads its before from the event"] = function()
    local hall = { oid = CALL_MOM, subject = "meet", labels = {}, event = { location = "hall" } }
    local answer = fixtures.decoded("edit-preview-fields.json")
    answer.oids, answer.changes[1].oid = { CALL_MOM }, CALL_MOM
    answer.changes[1].fields, answer.changes[1].location = { "location" }, "room"
    same(change_lines.reading(answer, { [CALL_MOM] = hall }, "location"), "location  hall → room")
  end,

  ["a focus mark reads as its day"] = function()
    local answer = fixtures.decoded("edit-preview-fields.json")
    answer.changes[1].fields = { "focus" }
    answer.changes[1].focus = { day = "2026-12-01", set_by = "owner", set_at = "2026-11-30T09:00:00Z" }
    same(change_lines.reading(answer, OBJECTS, "focus"), "focus  none → 2026-12-01")
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
