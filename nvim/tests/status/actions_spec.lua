local actions = require("damnit.status.actions")

local OID = ("2"):rep(40)

local function change(section)
  return { kind = "change", id = OID, oid = OID, section = section }
end

local function header(section)
  return { kind = "section", section = section }
end

local function argv(target, key)
  return actions.argv(target, key)
end

return {
  ["s on an unstaged change stages it"] = function()
    assert(vim.deep_equal(argv(change("unstaged"), "s"), { "add", OID }))
  end,

  ["s on the Unstaged or Waiting header stages all"] = function()
    assert(vim.deep_equal(argv(header("unstaged"), "s"), { "add", "-A" }))
    assert(vim.deep_equal(argv(header("waiting"), "s"), { "add", "-A" }))
  end,

  ["u on a staged change unstages it, on the Staged header unstages all"] = function()
    assert(vim.deep_equal(argv(change("staged"), "u"), { "reset", OID }))
    assert(vim.deep_equal(argv(header("staged"), "u"), { "reset" }))
  end,

  ["a accepts and x declines a waiting item or set by its hash"] = function()
    for _, kind in ipairs({ "waiting", "set" }) do
      local target = { kind = kind, id = "abc1234", section = "waiting" }
      assert(vim.deep_equal(argv(target, "a"), { "add", "abc1234" }), kind)
      assert(vim.deep_equal(argv(target, "x"), { "rm", "abc1234" }), kind)
    end
  end,

  ["a key on a row it does not fit gives no command"] = function()
    assert(argv(change("staged"), "s") == nil)
    assert(argv(change("unstaged"), "u") == nil)
    assert(argv(change("unstaged"), "a") == nil)
    assert(argv(header("staged"), "x") == nil)
    assert(argv(nil, "s") == nil)
  end,

  ["each key says which rows it fits"] = function()
    for _, key in ipairs({ "s", "u", "a", "x" }) do
      assert(actions.fits(key):find("^" .. key .. " "), actions.fits(key))
    end
  end,
}
