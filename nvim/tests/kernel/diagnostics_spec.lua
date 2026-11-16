local diagnostics = require("damnit.kernel.diagnostics")
local fixtures = require("helpers.fixtures")

local OAT_MILK, CALL_MOM = ("1"):rep(40), ("2"):rep(40)

local function refusal(name)
  return fixtures.decoded(name).error
end

local function scratch(lines)
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  return buf
end

return {
  ["a refusal lands on each row whose oid dam named"] = function()
    local oids = { [1] = CALL_MOM, [2] = OAT_MILK, [4] = "<id-1>" }
    local lines = diagnostics.rows_of(refusal("error-done-open-children.json"), oids)
    assert(vim.deep_equal(lines, { 1, 3 }), vim.inspect(lines))
  end,

  ["a refusal that names none of the targets lands on the rows it names and on the targets"] = function()
    local oids = { [1] = CALL_MOM, [2] = OAT_MILK }
    local err = { kind = "refused", message = "full", oids = { OAT_MILK } }
    local lines = diagnostics.rows_of(err, oids, { CALL_MOM })
    assert(vim.deep_equal(lines, { 0, 1 }), vim.inspect(lines))
  end,

  ["a refusal that names a target lands only on the rows it names"] = function()
    local oids = { [1] = CALL_MOM, [2] = OAT_MILK, [4] = "<id-1>" }
    local lines = diagnostics.rows_of(refusal("error-done-open-children.json"), oids, { OAT_MILK })
    assert(vim.deep_equal(lines, { 1, 3 }), vim.inspect(lines))
  end,

  ["a refusal that names no row lands on line 1"] = function()
    assert(vim.deep_equal(diagnostics.rows_of(refusal("error-expect.json"), { CALL_MOM }), { 0 }))
  end,

  ["in the task buffer the refusal lands on the header of the flag dam named in its message"] = function()
    local lines = { "subject: call mom", "due: 2026-09-25", "labels: " }
    assert(diagnostics.header_line(refusal("error-edit-due.json"), lines) == 1)
    local unlabels = { kind = "usage", message = "x", oids = {}, usage = { problem = "other", word = "--unlabels" } }
    assert(diagnostics.header_line(unlabels, lines) == 2)
    assert(diagnostics.header_line(refusal("error-done-open-children.json"), lines) == 0)
  end,

  ["show puts an ERROR with dam's message and suggested command, as lines under the row and no sign"] = function()
    local buf = scratch({ "one", "two" })
    local err = refusal("error-usage-selection.json")
    err.message = [[--priority cannot be "urgent"]]
    diagnostics.show(buf, err, { 1 })
    local shown = vim.diagnostic.get(buf)
    assert(#shown == 1 and shown[1].lnum == 1, vim.inspect(shown))
    assert(shown[1].severity == vim.diagnostic.severity.ERROR)
    assert(shown[1].message == require("damnit.kernel.error").text(err), shown[1].message)
    assert(shown[1].message:find("\nrun: ", 1, true), shown[1].message)
    local config = vim.diagnostic.config(nil, shown[1].namespace)
    assert(config.virtual_lines and not config.virtual_text and not config.signs, vim.inspect(config))
  end,

  ["clear takes the refusals off"] = function()
    local buf = scratch({ "one" })
    diagnostics.show(buf, refusal("error-expect.json"), { 0 })
    diagnostics.clear(buf)
    assert(#vim.diagnostic.get(buf) == 0)
  end,
}
