local brief = require("damnit.handoff.brief")
local fixtures = require("helpers.fixtures")

local OID = "4f3c2a1b9e8d7c6b5a4f3c2a1b9e8d7c6b5a4f3c"

local function task(fields)
  return vim.tbl_extend("force", { oid = OID, kind = "task", subject = "fix the login loop", path = "work/" }, fields)
end

local function equal(got, want)
  assert(got == want, ("got:\n%s\nwant:\n%s"):format(got, want))
end

return {
  ["the brief holds the subject, short oid, path, labels, due and body of dam show's task"] = function()
    local shown = fixtures.decoded("show.json")
    shown.oid = OID
    equal(
      brief.text(shown, nil),
      table.concat({
        "task: plant the tomatoes",
        "oid: 4f3c2a1",
        "path: home/",
        "labels: deep, garden",
        "due: 2026-09-25",
        "",
        "loosen the soil",
        "water after",
        "",
        "dam show " .. OID .. " -o json",
      }, "\n")
    )
  end,

  ["a note is kept as the person typed it, above the line to read the task again"] = function()
    equal(
      brief.text(task({}), "start with the redirect test"),
      table.concat({
        "task: fix the login loop",
        "oid: 4f3c2a1",
        "path: work/",
        "",
        "note: start with the redirect test",
        "",
        "dam show " .. OID .. " -o json",
      }, "\n")
    )
  end,

  ["fields dam left out leave no empty lines behind"] = function()
    equal(
      brief.text(task({ labels = {}, body = "", task = { done = false } }), ""),
      table.concat(
        { "task: fix the login loop", "oid: 4f3c2a1", "path: work/", "", "dam show " .. OID .. " -o json" },
        "\n"
      )
    )
  end,
}
