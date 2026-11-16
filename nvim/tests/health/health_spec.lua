local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local plugin_dir = vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2))))

local function checked(fake_reply)
  local app, fake = helper.new({ handoff = { host = "clipboard" } })
  for verb, reply in pairs(fake_reply) do
    fake.reply(verb, reply)
  end
  local damnit = package.loaded.damnit
  package.loaded.damnit = {
    app = function()
      return app
    end,
  }
  vim.opt.rtp:prepend(plugin_dir)
  local ok, err = pcall(vim.cmd, "checkhealth damnit")
  package.loaded.damnit = damnit
  assert(ok, err)
  return table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), "\n"), fake
end

local function holds(text, part)
  assert(text:find(part, 1, true), ("%q not in:\n%s"):format(part, text))
end

local function with_issue()
  local doc = fixtures.decoded("doctor.json")
  doc.checks[2] = { name = "database", result = "issue", detail = "unreadable", advice = "move it aside", rows = {} }
  return { code = 1, stdout = vim.json.encode(doc), stderr = "" }
end

return {
  [":checkhealth damnit shows dam's version, each doctor check and the hand-off host"] = function()
    local text, fake = checked({ doctor = fixtures.answer("doctor.json") })
    assert(vim.deep_equal(fake.of("doctor")[1].argv, { "dam", "doctor", "-o", "json" }), vim.inspect(fake.of("doctor")))
    holds(text, "OK dam 0.2.0, inside >=0.2.0 <0.3.0")
    holds(text, "OK config: both files are valid")
    holds(text, "OK database: none yet; dam doctor --fix creates it")
    holds(text, "    fake (ok): dam-remote-fake answered")
    holds(text, "OK hand-off goes to the clipboard")
  end,

  ["an issue, which makes dam doctor exit 1, shows as an error with dam's advice"] = function()
    local text = checked({ doctor = with_issue() })
    holds(text, "ERROR database: unreadable")
    holds(text, "- move it aside")
    holds(text, "OK config: both files are valid")
  end,

  ["no dam is an error naming the setting and runs no doctor"] = function()
    local missing = { code = -1, stdout = "", stderr = "ENOENT", missing = true }
    local text, fake = checked({ ["--version"] = missing, doctor = fixtures.answer("doctor.json") })
    holds(text, 'ERROR cannot run "dam"; install dam or set setup({ dam = "/path/to/dam" })')
    assert(#fake.of("doctor") == 0, vim.inspect(fake.of("doctor")))
  end,
}
