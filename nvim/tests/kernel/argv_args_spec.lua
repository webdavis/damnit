local argv = require("damnit.kernel.argv")

local FLAGS = {
  { name = "--due", value = true, help = "" },
  { name = "--labels", value = true, help = "" },
  { name = "--subject", value = true, help = "" },
  { name = "--preview", value = false, help = "" },
}

local function args(line)
  return argv.args(line, FLAGS)
end

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["the words after a flag that takes a value are its value, up to the next flag"] = function()
    same(
      args("call the bank --due next fri --labels home"),
      { "call", "the", "bank", "--due", "next fri", "--labels", "home" }
    )
  end,

  ["a quoted value wins and ends the value"] = function()
    same(args([[--due "next fri" soon]]), { "--due", "next fri", "soon" })
    same(args([[--subject 'say "hi"']]), { "--subject", [[say "hi"]] })
  end,

  ["a flag that takes no value takes none"] = function()
    same(args("--preview next fri"), { "--preview", "next", "fri" })
  end,

  ["an unquoted word that looks like a flag but is none of the command's ends the value and goes to dam as typed"] = function()
    same(args("--due next --force fri"), { "--due", "next", "--force", "fri" })
  end,

  ["a quoted word that looks like a flag stays in the value"] = function()
    same(args([[--subject fix "--due" parsing]]), { "--subject", "fix --due parsing" })
  end,

  ["a flag of the command typed in quotes is a word, not a flag"] = function()
    same(args([[buy "--due" --due fri]]), { "buy", "--due", "--due", "fri" })
  end,
}
