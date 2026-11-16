local text = require("damnit.capture.text")

local LUA = { commentstring = "-- %s", comments = "s:--[[,m: ,e:]],:--" }
local C = { commentstring = "/* %s */", comments = "s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-" }
local SHELL = { commentstring = "# %s", comments = "b:#" }

local function equal(got, want)
  assert(got == want, ("got %q, want %q"):format(tostring(got), tostring(want)))
end

return {
  ["a comment line loses its leader and its TODO marker"] = function()
    equal(text.subject({ "  -- TODO: rename the queue lanes" }, LUA), "rename the queue lanes")
    equal(text.subject({ "# FIXME(sam) handle a missing file" }, SHELL), "handle a missing file")
    equal(text.subject({ "// XXX: drop this branch" }, C), "drop this branch")
    equal(text.subject({ "-- NOTE keep the order" }, LUA), "keep the order")
  end,

  ["a word that only starts like a marker stays"] = function()
    equal(text.subject({ "-- TODOS are listed here" }, LUA), "TODOS are listed here")
  end,

  ["a block comment loses both ends"] = function()
    equal(text.subject({ "/* TODO: free the buffer */" }, C), "free the buffer")
    equal(text.subject({ "/*", " * TODO: free the buffer", " * before returning", " */" }, C), "free the buffer before returning")
    equal(text.subject({ "--[[ TODO: split this file ]]" }, LUA), "split this file")
  end,

  ["a selection joins into one subject"] = function()
    equal(text.subject({ "-- TODO: rename the", "", "--   queue lanes" }, LUA), "rename the queue lanes")
  end,

  ["a line that is not a comment is kept as it reads"] = function()
    equal(text.subject({ "  local lanes = {}  " }, LUA), "local lanes = {}")
  end,

  ["an empty line gives an empty subject"] = function()
    equal(text.subject({ "   " }, LUA), "")
    equal(text.subject({ "-- TODO:" }, LUA), "")
  end,

  ["the body names the file relative to dam's folder and the line"] = function()
    equal(text.place("/work/garden/src/beds.lua", "/work/garden/src", 12), "beds.lua:12")
    equal(text.place("/work/garden/src/beds.lua", "/work/garden", 3), "src/beds.lua:3")
    equal(text.place("/elsewhere/notes.md", "/work/garden", 1), "/elsewhere/notes.md:1")
    equal(text.place("", "/work/garden", 4), nil)
  end,

  ["a quoted word keeps quotes, backslashes and dashes as written"] = function()
    equal(text.quoted("don't --skip"), [["don't --skip"]])
    equal(text.quoted([[say "hi" \o/]]), [["say \"hi\" \\o/"]])
  end,
}
