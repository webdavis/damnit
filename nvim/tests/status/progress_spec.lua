local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")

local function spied()
  local seen = {}
  vim.api.nvim_create_autocmd("Progress", {
    group = "DamSpec",
    pattern = "dam",
    callback = function(ev)
      table.insert(seen, ev.data.status)
    end,
  })
  return seen
end

local function pushed_after_yes(fake, reply)
  fake.reply("push", function(argv)
    return vim.tbl_contains(argv, "--preview") and fixtures.answer("push-preview.json") or reply
  end)
  buffer.feed("P")
  buffer.wait_for(function()
    return vim.bo.filetype == "damconfirm"
  end)
  buffer.feed("y")
end

return {
  ["P shows the push running then success"] = function()
    local _, fake = status.opened()
    local seen = spied()
    pushed_after_yes(
      fake,
      { code = 0, stdout = fixtures.read("push.json"), stderr = fixtures.read("progress-push.txt") }
    )
    buffer.wait_for(function()
      return #seen == 2
    end)
    assert(vim.deep_equal(seen, { "running", "success" }), vim.inspect(seen))
  end,

  ["a failed push ends its progress as failed and shows dam's message"] = function()
    local _, fake = status.opened()
    local seen = spied()
    local notes = buffer.capture_notes()
    pushed_after_yes(fake, { code = 1, stdout = "", stderr = fixtures.read("progress-push-failed.txt") })
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(vim.deep_equal(seen, { "running", "failed" }), vim.inspect(seen))
    assert(notes[1].level == vim.log.levels.ERROR, vim.inspect(notes))
    assert(notes[1].text == "the remote refused: fake is rate limiting this account; retry after 42s", notes[1].text)
  end,

  [":Dam cancel ends a running pull's progress as cancelled"] = function()
    local _, fake = status.opened()
    local seen = spied()
    fake.reply("pull", { code = 0, stdout = fixtures.read("pull.json"), stderr = "", hold = true })
    vim.cmd("Dam pull")
    buffer.wait_for(function()
      return #fake.of("pull") == 1
    end)
    fake.line(fake.of("pull")[1], vim.split(fixtures.read("progress-pull.txt"), "\n")[1])
    buffer.wait_for(function()
      return #seen == 1
    end)
    vim.cmd("Dam cancel")
    vim.wait(30)
    assert(vim.deep_equal(seen, { "running", "cancel" }), vim.inspect(seen))
    assert(fake.of("pull")[1].killed, vim.inspect(fake.of("pull")))
  end,
}
