local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")
local ui = require("helpers.ui")

local EXIT_REFUSED = 4

local function held(fake, verb)
  fake.reply(verb, { code = 0, stdout = fixtures.read(verb .. ".json"), stderr = "", hold = true })
end

local function cancelled_by_dam_cancel(fake, key, verb)
  held(fake, verb)
  buffer.feed(key)
  buffer.wait_for(function()
    return #fake.of(verb) == 1
  end)
  vim.cmd("Dam cancel")
  vim.wait(30)
  assert(fake.of(verb)[1].killed, vim.inspect(fake.of(verb)))
end

return {
  ["c sends dam commit with the typed message"] = function()
    local _, fake = status.opened()
    ui.answering({ input = "tidy the week" }, function(prompts)
      buffer.feed("c")
      assert(prompts[1] == "commit message: ", vim.inspect(prompts))
    end)
    local sent = status.sent(fake, "commit")
    assert(vim.deep_equal(sent, { "commit", "-m", "tidy the week" }), vim.inspect(sent))
  end,

  ["an empty commit message sends nothing"] = function()
    local _, fake = status.opened()
    for _, typed in ipairs({ "", "  " }) do
      ui.answering({ input = typed }, function()
        buffer.feed("c")
      end)
    end
    ui.answering({}, function()
      buffer.feed("c")
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
    assert(vim.deep_equal(fake.of("push")[1].argv, { "dam", "push", "--progress", "-o", "json" }))
    assert(vim.deep_equal(fake.of("pull")[1].argv, { "dam", "pull", "--progress", "-o", "json" }))
  end,

  ["a push says what each remote took"] = function()
    local _, fake = status.opened()
    local notes = buffer.capture_notes()
    buffer.feed("P")
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text == "pushed: fake sent 2, failed 0", vim.inspect(notes))
    assert(#fake.of("push") == 1)
  end,

  ["a failed push shows dam's message"] = function()
    local _, fake = status.opened()
    fake.reply("push", fixtures.refusal("error-refused.json", EXIT_REFUSED))
    local notes = buffer.capture_notes()
    buffer.feed("P")
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].level == vim.log.levels.ERROR, vim.inspect(notes))
    assert(notes[1].text:find(fixtures.decoded("error-refused.json").error.message, 1, true), vim.inspect(notes))
  end,
}
