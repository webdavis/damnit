local actions = require("damnit.list.actions")

local A, B = ("a"):rep(40), ("b"):rep(40)

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["x is dam done on the targets"] = function()
    same(actions.batch("x", { A, B }), { command = "done", oids = { A, B }, flags = {} })
  end,

  ["X reopens, p1 to p4 set the priority, s stages"] = function()
    same(actions.batch("X", { A }).flags, { "--no-done" })
    same(actions.batch("p3", { A }).flags, { "--priority", "3" })
    same(actions.batch("s", { A }).command, "add")
  end,

  ["dd removes and U sets back"] = function()
    same(actions.batch("dd", { A }).command, "rm")
    same(actions.batch("U", { A }).command, "restore")
  end,

  ["a change key's answer is read with its command's flags, a value of several words kept whole"] = function()
    local flags_of = function(command)
      assert(command == "edit", command)
      return { { name = "--due", value = true, help = "" }, { name = "--labels", value = true, help = "" } }
    end
    same(actions.batch("cd", { A }, "--due next fri --labels home", flags_of).flags, {
      "--due",
      "next fri",
      "--labels",
      "home",
    })
  end,

  ["a change key's answer is its flags, split as a shell would"] = function()
    same(
      actions.batch("cd", { A }, [[--due "next fri"]]),
      { command = "edit", oids = { A }, flags = { "--due", "next fri" } }
    )
    same(actions.batch("cm", { A }, "--path work/").command, "mv")
    same(actions.batch("gx", { A }, "--force"), { command = "done", oids = { A }, flags = { "--force" } })
  end,

  ["one target is sent at once; two or more, dd and U ask first"] = function()
    assert(not actions.confirms("x", { A }))
    assert(actions.confirms("x", { A, B }))
    assert(actions.confirms("dd", { A }))
    assert(actions.confirms("U", { A }))
  end,

  ["a change key's prompt is prefilled with its flag"] = function()
    same(actions.prompt("cd"), { title = "dam edit", prefill = "--due ", field = "due" })
    same(actions.prompt("cP"), { title = "dam mv", prefill = "--parent ", field = "path" })
    same(actions.prompt("cc"), { title = "dam edit", prefill = "" })
    same(actions.prompt("gx"), { title = "dam done", prefill = "" })
    assert(actions.prompt("x") == nil)
  end,

  ["cs is prefilled with the row's subject, quoted"] = function()
    local prompt = actions.prompt("cs", { subject = [[say "hi" to mom]] })
    same(prompt.prefill, [[--subject "say \"hi\" to mom"]])
    assert(prompt.one, "cs takes more than one row")
  end,

  ["the echo after a write counts the targets in the key's words"] = function()
    same(actions.said("x", 3), "dam: 3 done")
    same(actions.said("X", 1), "dam: 1 reopened")
    same(actions.said("cd", 2), "dam: 2 changed")
    same(actions.said("dd", 2), "dam: 2 removed")
  end,

  ["every key has a group and a description for g?"] = function()
    for _, key in ipairs(actions.keys()) do
      local spec = actions.spec(key)
      assert(spec.group and spec.desc, key)
    end
    assert(#actions.keys() == 21, #actions.keys())
  end,
}
