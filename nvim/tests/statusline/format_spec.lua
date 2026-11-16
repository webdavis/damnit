local fixtures = require("helpers.fixtures")
local format = require("damnit.statusline.format")

local SIGNS = require("damnit.kernel.options").defaults().signs
local SOME = { overdue = 2, waiting = 1, unstaged = 3, staged = 1, unpushed = 2 }

local function equal(got, want)
  assert(got == want, ("got %q, want %q"):format(got, want))
end

return {
  ["the counts read from dam's status and its overdue list"] = function()
    local counts = format.counts(fixtures.decoded("list-overdue.json"), fixtures.decoded("status.json"))
    local want = { overdue = 1, waiting = 2, unstaged = 1, staged = 1, unpushed = 2 }
    assert(vim.deep_equal(counts, want), vim.inspect(counts))
  end,

  ["each count reads as its sign and number, zeros left out"] = function()
    equal(format.text(SOME, SIGNS), "!2 ?1 M3 S1 ^2")
    equal(format.text({ overdue = 0, waiting = 0, unstaged = 4, staged = 0, unpushed = 0 }, SIGNS), "M4")
  end,

  ["the signs option changes the characters"] = function()
    local signs = vim.tbl_extend("force", SIGNS, { overdue = "x", waiting = "%" })
    equal(format.text(SOME, signs), "x2 %%1 M3 S1 ^2")
  end,

  ["everything zero reads as empty"] = function()
    equal(format.text({ overdue = 0, waiting = 0, unstaged = 0, staged = 0, unpushed = 0 }, SIGNS), "")
    equal(format.text(nil, SIGNS), "")
  end,

  ["the highlight form wraps each count in its Dam group"] = function()
    local text = format.text({ overdue = 2, waiting = 0, unstaged = 3, staged = 0, unpushed = 0 }, SIGNS, true)
    equal(text, "%#DamOverdue#!2%* %#DamChanged#M3%*")
  end,
}
