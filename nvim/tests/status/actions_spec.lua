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

  ["a accepts and dd declines a waiting item or set by its hash"] = function()
    for _, kind in ipairs({ "waiting", "set" }) do
      local target = { kind = kind, id = "abc1234", section = "waiting" }
      assert(vim.deep_equal(argv(target, "a"), { "add", "abc1234" }), kind)
      assert(vim.deep_equal(argv(target, "dd"), { "rm", "abc1234" }), kind)
    end
  end,

  ["a key on a row it does not fit gives no command"] = function()
    assert(argv(change("staged"), "s") == nil)
    assert(argv(change("unstaged"), "u") == nil)
    assert(argv(change("unstaged"), "a") == nil)
    assert(argv(header("staged"), "dd") == nil)
    assert(argv(nil, "s") == nil)
  end,

  ["each key says which rows it fits"] = function()
    for _, key in ipairs({ "s", "u", "a", "dd", "U" }) do
      assert(actions.fits(key):find("^" .. key .. " "), actions.fits(key))
    end
  end,

  ["U sets an unstaged or staged change back with dam restore, and fits no other row"] = function()
    assert(vim.deep_equal(argv(change("unstaged"), "U"), { "restore", OID }))
    assert(vim.deep_equal(argv(change("staged"), "U"), { "restore", OID }))
    assert(argv(header("unstaged"), "U") == nil)
  end,

  ["dd's confirm rows are the waiting item or set as dam printed it"] = function()
    local doc = require("helpers.fixtures").decoded("status.json")
    local rows, summary = actions.declined(doc, { kind = "waiting", id = doc.waiting[1].hash })
    local want = { { op = "new", subject = "call the plumber", detail = "from tool" } }
    assert(vim.deep_equal(rows, want) and summary == "1 waiting item would be declined", vim.inspect(rows))
  end,
}
