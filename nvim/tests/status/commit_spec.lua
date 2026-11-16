local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")
local ui = require("helpers.ui")

local EXIT_REFUSED = 4

local function held(fake, verb)
  local hold = { code = 0, stdout = fixtures.read(verb .. ".json"), stderr = "", hold = true }
  fake.reply(verb, function(argv)
    return vim.tbl_contains(argv, "--preview") and fixtures.answer("push-preview.json") or hold
  end)
end

local function confirmed_push()
  buffer.feed("P")
  buffer.wait_for(function()
    return vim.bo.filetype == "damconfirm"
  end)
  buffer.feed("y")
end

local function cancelled_by_dam_cancel(fake, key, verb)
  held(fake, verb)
  if key == "P" then
    confirmed_push()
  else
    buffer.feed(key)
  end
  buffer.wait_for(function()
    return #fake.of(verb) == (verb == "push" and 2 or 1)
  end)
  vim.cmd("Dam cancel")
  vim.wait(30)
  assert(fake.of(verb)[#fake.of(verb)].killed, vim.inspect(fake.of(verb)))
end

return {
  ["cc sends dam commit with the typed message"] = function()
    local _, fake = status.opened()
    ui.answering({ input = "tidy the week" }, function(prompts)
      buffer.feed("cc")
      assert(prompts[1] == "commit message: ", vim.inspect(prompts))
    end)
    local sent = status.sent(fake, "commit")
    assert(vim.deep_equal(sent, { "commit", "-m", "tidy the week" }), vim.inspect(sent))
  end,

  ["an empty commit message sends nothing"] = function()
    local _, fake = status.opened()
    for _, typed in ipairs({ "", "  " }) do
      ui.answering({ input = typed }, function()
        buffer.feed("cc")
      end)
    end
    ui.answering({}, function()
      buffer.feed("cc")
    end)
    status.nothing_sent(fake, "commit")
  end,

  [":Dam commit sends its words as the message"] = function()
    local _, fake = status.defined()
    vim.cmd("Dam commit tidy the week")
    local sent = status.sent(fake, "commit")
    assert(vim.deep_equal(sent, { "commit", "-m", "tidy the week" }), vim.inspect(sent))
  end,

  [":Dam push and :Dam pull name the remote and show progress"] = function()
    local _, fake = status.defined()
    vim.cmd("Dam push fake")
    vim.cmd("Dam pull fake")
    assert(
      vim.deep_equal(status.sent(fake, "push"), { "push", "fake", "--progress" }),
      vim.inspect(status.sent(fake, "push"))
    )
    assert(
      vim.deep_equal(status.sent(fake, "pull"), { "pull", "fake", "--progress" }),
      vim.inspect(status.sent(fake, "pull"))
    )
  end,

  ["P sends dam push --progress and F dam pull --progress through the queue"] = function()
    local _, fake = status.opened()
    cancelled_by_dam_cancel(fake, "P", "push")
    cancelled_by_dam_cancel(fake, "F", "pull")
    local push = fake.of("push")[2].argv
    assert(vim.deep_equal(push, { "dam", "push", "--progress", "-o", "json" }), vim.inspect(fake.of("push")))
    assert(vim.deep_equal(fake.of("pull")[1].argv, { "dam", "pull", "--progress", "-o", "json" }))
  end,

  ["a push says what each remote took"] = function()
    local _, fake = status.opened()
    local notes = buffer.capture_notes()
    confirmed_push()
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text == "pushed: fake sent 2, failed 0", vim.inspect(notes))
    assert(#fake.of("push") == 2)
  end,

  ["a refused push preview shows dam's message once, in the window"] = function()
    local app, fake = status.opened()
    fake.reply("push", fixtures.refusal("error-refused.json", EXIT_REFUSED))
    local notes = buffer.capture_notes()
    buffer.feed("P")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) > 0
    end)
    local message = fixtures.decoded("error-refused.json").error.message
    assert(vim.diagnostic.get(0)[1].message:find(message, 1, true), vim.inspect(vim.diagnostic.get(0)))
    vim.wait(20)
    assert(#notes == 0 and #app.say:log() == 1, vim.inspect({ notes, app.say:log() }))
  end,

  ["P shows what dam push --preview would send, and n pushes nothing"] = function()
    local _, fake = status.opened()
    buffer.feed("P")
    buffer.wait_for(function()
      return vim.bo.filetype == "damconfirm"
    end)
    assert(vim.deep_equal(fake.of("push")[1].argv, { "dam", "push", "--preview", "-o", "json" }))
    assert(vim.tbl_contains(buffer.lines(), "1 would be sent, 0 held"), vim.inspect(buffer.lines()))
    buffer.feed("n")
    vim.wait(30)
    assert(#fake.of("push") == 1, vim.inspect(fake.of("push")))
  end,
}
