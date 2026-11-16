local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local log = require("helpers.log")
local reverted = require("damnit.log.reverted")
local status = require("helpers.status")

local ERRAND = "<id-2>  <now>  plan the errand  1 change   buy oat milk"
local EXIT_REFUSED = 4

local function confirming()
  status.on(ERRAND)
  buffer.feed("R")
  buffer.wait_for(function()
    return vim.bo.filetype == "damconfirm"
  end)
end

return {
  ["R lists the commit's changes from dam show and says they come back as working changes"] = function()
    local _, fake = log.opened()
    confirming()
    assert(vim.deep_equal(fake.of("show")[1].argv, { "dam", "show", "<id-2>", "-o", "json" }))
    local expected = { "update  buy oat milk  labels, priority, due", "", "1 change comes back as a working change" }
    assert(vim.deep_equal(buffer.lines(), expected), vim.inspect(buffer.lines()))
    buffer.feed("n")
    status.nothing_sent(fake, "revert")
  end,

  ["y sends dam revert, says what came back and opens the status window"] = function()
    local _, fake = log.opened()
    local notes = buffer.capture_notes()
    confirming()
    buffer.feed("y")
    assert(vim.deep_equal(status.sent(fake, "revert"), { "revert", "<id-2>" }), vim.inspect(fake.of("revert")))
    buffer.wait_for(function()
      return vim.bo.filetype == "damstatus"
    end)
    local said = vim.tbl_map(function(note)
      return note.text
    end, notes)
    assert(
      vim.tbl_contains(said, "1 change came back; kept as changed since: buy oat milk priority"),
      vim.inspect(said)
    )
  end,

  ["a refused revert shows dam's words once, on that commit's row, and stays in the log"] = function()
    local app, fake = log.opened()
    fake.reply("revert", fixtures.refusal("error-revert-pulled.json", EXIT_REFUSED))
    local notes = buffer.capture_notes()
    confirming()
    buffer.feed("y")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) > 0
    end)
    local found = vim.diagnostic.get(0)
    assert(
      #found == 1 and found[1].lnum == 1 and found[1].message:find("came from dam pull", 1, true),
      vim.inspect(found)
    )
    assert(vim.bo.filetype == "damlog" and #notes == 0 and #app.say:log() == 1, vim.inspect(notes))
  end,

  ["the message counts every change that came back and names nothing kept when nothing was"] = function()
    local answer = {
      reverted = { { oid = ("1"):rep(40), fields = { "due" } }, { oid = ("2"):rep(40), fields = { "subject" } } },
      kept = {},
    }
    assert(reverted.text(answer, { changes = {} }) == "2 changes came back")
  end,

  ["a kept field of an object the commit does not name is named by its short oid"] = function()
    local answer = { reverted = {}, kept = { { oid = ("3"):rep(40), fields = { "due", "labels" } } } }
    local text = reverted.text(answer, { changes = {} })
    assert(text == "0 changes came back; kept as changed since: 3333333 due, labels", text)
  end,
}
