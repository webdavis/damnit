local process = require("damnit.kernel.process")
local wait_for = require("helpers.buffer").wait_for

local SCRIPT = "printf 'one\\n' >&2; sleep 0.05; printf 'two\\nthree' >&2; printf '{}'"

local function ran(on_line)
  local got = { lines = {} }
  process.run({ "sh", "-c", SCRIPT }, {
    cwd = vim.fn.getcwd(),
    on_line = on_line and function(line)
      table.insert(got.lines, { line = line, exited = got.exit ~= nil })
    end,
  }, function(exit)
    got.exit = exit
  end)
  wait_for(function()
    return got.exit ~= nil
  end)
  return got
end

return {
  ["standard error comes line by line as it arrives and whole at exit"] = function()
    local got = ran(true)
    local lines = vim.tbl_map(function(each)
      return each.line
    end, got.lines)
    assert(vim.deep_equal(lines, { "one", "two", "three" }), vim.inspect(got.lines))
    assert(not got.lines[1].exited, "the first line came only at exit")
    assert(got.exit.stderr == "one\ntwo\nthree" and got.exit.stdout == "{}", vim.inspect(got.exit))
  end,

  ["without a line reader standard error comes whole at exit"] = function()
    local got = ran(false)
    assert(#got.lines == 0 and got.exit.stderr == "one\ntwo\nthree", vim.inspect(got))
  end,
}
