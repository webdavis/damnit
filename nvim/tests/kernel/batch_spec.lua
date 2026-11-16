local batch = require("damnit.kernel.batch")

local A, B, C = ("a"):rep(40), ("b"):rep(40), ("c"):rep(40)

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["one oid goes inline before the flags"] = function()
    local args, stdin = batch.call({ command = "edit", oids = { A }, flags = { "--priority", "2" } })
    same(args, { "edit", A, "--priority", "2" })
    assert(stdin == nil, stdin)
  end,

  ["two or more go as - with the oids on stdin, one per line, and --expect N"] = function()
    local args, stdin = batch.call({ command = "done", oids = { A, B }, flags = {} })
    same(args, { "done", "-", "--expect", "2" })
    assert(stdin == A .. "\n" .. B .. "\n", stdin)
  end,

  ["--preview comes last when asked"] = function()
    same(batch.call({ command = "rm", oids = { A, B, C }, flags = {}, preview = true }), {
      "rm",
      "-",
      "--expect",
      "3",
      "--preview",
    })
    same(batch.call({ command = "done", oids = { A }, flags = {}, preview = true }), { "done", A, "--preview" })
  end,

  ["--as-one joins a many-oid call only under a token"] = function()
    local call = { command = "edit", oids = { A, B }, flags = { "--due", "fri" }, token = true }
    same(batch.call(call), { "edit", "-", "--due", "fri", "--expect", "2", "--as-one" })
    call.oids = { A }
    same(batch.call(call), { "edit", A, "--due", "fri" })
  end,

  ["restore and add take every oid as a word and no --expect"] = function()
    local args, stdin = batch.call({ command = "restore", oids = { A, B }, flags = {}, preview = true, token = true })
    same(args, { "restore", A, B, "--preview" })
    assert(stdin == nil, stdin)
    same(batch.call({ command = "add", oids = { A, B, C }, flags = {} }), { "add", A, B, C })
  end,
}
