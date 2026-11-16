local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local OID = "abcdef1"
local EXIT_REFUSED = 4

local function shown(reply)
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.task").new(app) })
  fake.reply("show", fixtures.answer(reply or "show.json"))
  fake.reply("edit", fixtures.answer("edit.json"))
  vim.cmd("Dam task " .. OID)
  buffer.wait_for(function()
    return buffer.lines()[1]:find("^subject:") ~= nil
  end)
  return app, fake
end

local function set_line(n, text)
  vim.api.nvim_buf_set_lines(0, n - 1, n, false, { text })
end

local function written(fake)
  local notes = buffer.capture_notes()
  vim.cmd.write()
  buffer.wait_for(function()
    return #fake.of("edit") > 0 or #notes > 0 or not vim.bo.modified
  end)
  vim.wait(20)
  return fake.of("edit")[1], notes
end

local function sent_flags(call)
  assert(call, "dam edit never ran")
  return vim.list_slice(call.argv, 4, #call.argv - 2)
end

local function equal(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["an untouched write sends nothing and clears modified"] = function()
    local _, fake = shown()
    vim.bo.modified = true
    local call = written(fake)
    assert(call == nil, vim.inspect(call))
    assert(not vim.bo.modified, "still modified")
  end,

  ["changing the due line sends --due with the typed text"] = function()
    local _, fake = shown()
    set_line(2, "due: next friday")
    local call = written(fake)
    equal(vim.list_slice(call.argv, 1, 3), { "dam", "edit", OID })
    equal(sent_flags(call), { "--due", "next friday" })
  end,

  ["clearing the due line sends --no-due"] = function()
    local _, fake = shown()
    set_line(2, "due:")
    equal(sent_flags(written(fake)), { "--no-due" })
  end,

  ["a changed body is sent as --body"] = function()
    local _, fake = shown()
    vim.api.nvim_buf_set_lines(0, 9, -1, false, { "dig first" })
    equal(sent_flags(written(fake)), { "--body", "dig first" })
  end,

  ["an event sends the flags of its start, end and location"] = function()
    local _, fake = shown("show-event.json")
    set_line(2, "start: tomorrow 10:00")
    set_line(4, "location:")
    equal(sent_flags(written(fake)), { "--start", "tomorrow 10:00", "--no-location" })
  end,

  ["a write redraws from dam's answer"] = function()
    local _, fake = shown()
    set_line(2, "due: oct 1")
    written(fake)
    buffer.wait_for(function()
      return buffer.lines()[2] == "due: 2026-10-01"
    end)
    assert(not vim.bo.modified, "still modified after dam's answer")
  end,

  ["a header that cannot be read keeps the buffer modified and names the line"] = function()
    local _, fake = shown()
    set_line(4, "urgency: high")
    local call, notes = written(fake)
    assert(call == nil, vim.inspect(call))
    assert(vim.bo.modified, "no longer modified")
    assert(notes[1] and notes[1].text:find("line 4", 1, true), vim.inspect(notes))
  end,

  ["a refused write keeps the buffer modified and shows dam's message"] = function()
    local _, fake = shown()
    fake.reply("edit", fixtures.refusal("error-refused-edit.json", EXIT_REFUSED))
    set_line(6, "category: effort=deep, effort=light")
    local notes = buffer.capture_notes()
    vim.cmd.write()
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) > 0
    end)
    assert(vim.diagnostic.get(0)[1].message:find("allows one value", 1, true), vim.inspect(vim.diagnostic.get(0)))
    assert(#notes == 0, vim.inspect(notes))
    assert(vim.bo.modified, "no longer modified")
    assert(buffer.lines()[6] == "category: effort=deep, effort=light", vim.inspect(buffer.lines()))
  end,

  ["a token's write says it waits for the owner"] = function()
    local _, fake = shown()
    fake.reply("edit", fixtures.answer("sent.json"))
    set_line(2, "due: oct 1")
    local notes = buffer.capture_notes()
    vim.cmd.write()
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text:find("waiting for the owner", 1, true), vim.inspect(notes))
    assert(not vim.bo.modified, "still modified after the item was sent")
    assert(buffer.lines()[2] == "due: oct 1", vim.inspect(buffer.lines()))
  end,
}
