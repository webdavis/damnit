local fixtures = require("helpers.fixtures")
local report = require("damnit.health.report")

local function equal(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

local function check(fields)
  return vim.tbl_extend("force", { name = "sync", result = "ok", detail = "", rows = {} }, fields)
end

local function doctor(checks)
  return { version = "0.2.0", command = "doctor", checks = checks, needs_attention = {} }
end

return {
  ["a supported dam is ok and names the range"] = function()
    equal(report.dam("0.2.0", nil), { level = "ok", text = "dam 0.2.0, inside >=0.2.0 <0.3.0" })
  end,

  ["a dam outside the range is an error saying what to do"] = function()
    equal(report.dam("0.9.1", nil), {
      level = "error",
      text = "dam 0.9.1 is outside >=0.2.0 <0.3.0; update dam or the plugin",
    })
  end,

  ["no dam is an error with the plugin's own message"] = function()
    local err = { kind = "plugin", message = 'cannot run "dam"; install dam', oids = {} }
    equal(report.dam(nil, err), { level = "error", text = 'cannot run "dam"; install dam' })
  end,

  ["each doctor check becomes one line at the level of its result"] = function()
    local levels = vim.tbl_map(
      function(item)
        return item.level
      end,
      report.checks(doctor({
        check({ result = "ok" }),
        check({ result = "warning" }),
        check({ result = "issue" }),
        check({ result = "not_run" }),
      }))
    )
    equal(levels, { "ok", "warn", "error", "info" })
  end,

  ["a warning or issue keeps dam's advice as advice"] = function()
    local items =
      report.checks(doctor({ check({ result = "issue", detail = "broken", advice = "run dam doctor --fix" }) }))
    equal(items, { { level = "error", text = "sync: broken", advice = "run dam doctor --fix" } })
  end,

  ["an ok check keeps dam's advice in its text"] = function()
    local items = report.checks(fixtures.decoded("doctor.json"))
    equal(items[2], { level = "ok", text = "database: none yet; dam doctor --fix creates it" })
  end,

  ["a check's rows are indented lines under it"] = function()
    local items = report.checks(fixtures.decoded("doctor.json"))
    equal(items[4], {
      level = "ok",
      text = "credentials: 1 remote: 1 ok\n  fake (ok): has its credential, not tested; run dam test credentials",
    })
  end,

  ["the host line names where a hand-off goes"] = function()
    equal(report.host("auto", "rex", nil, nil), { level = "ok", text = "hand-off goes to rex" })
    equal(report.host("clipboard", "clipboard", nil, "rex is not on PATH"), {
      level = "ok",
      text = "hand-off goes to the clipboard",
    })
    equal(report.host("auto", "clipboard", nil, "rex is not on PATH"), {
      level = "info",
      text = "hand-off goes to the clipboard; rex is not on PATH",
    })
    equal(report.host("rex", "rex", "rex is not on PATH", "rex is not on PATH"), {
      level = "warn",
      text = "hand-off falls back to the clipboard; rex is not on PATH",
      advice = 'run Neovim in a Rex pane, or set handoff = { host = "clipboard" }',
    })
  end,
}
