local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local wait_for = require("helpers.buffer").wait_for

local function spied()
  local seen = {}
  vim.api.nvim_create_autocmd("Progress", {
    group = "DamSpec",
    pattern = "dam",
    callback = function(ev)
      table.insert(seen, ev.data)
    end,
  })
  return seen
end

local function texts(seen)
  return vim.tbl_map(function(data)
    return table.concat(data.text)
  end, seen)
end

local function statuses(seen)
  return vim.tbl_map(function(data)
    return data.status
  end, seen)
end

local function replied(fake, stderr, code, hold)
  fake.reply("push", { code = code or 0, stdout = fixtures.read("push.json"), stderr = stderr, hold = hold })
end

local function push(app, progress)
  return helper.call(app, { args = { "push" }, cwd = "/", progress = progress })
end

local function started(fake, seen)
  replied(fake, "", 0, true)
  local got = push(fake.app, true)
  wait_for(function()
    return #fake.of("push") == 1
  end)
  fake.line(fake.of("push")[1], vim.split(fixtures.read("progress-push.txt"), "\n")[1])
  wait_for(function()
    return #seen == 1
  end)
  return got
end

return {
  ["a call with progress sends --progress"] = function()
    local app, fake = helper.new()
    replied(fake, "")
    push(app, true)
    wait_for(function()
      return #fake.of("push") == 1
    end)
    assert(vim.deep_equal(fake.of("push")[1].argv, { "dam", "push", "--progress", "-o", "json" }))
  end,

  ["a call without progress sends no --progress"] = function()
    local app, fake = helper.new()
    replied(fake, "")
    push(app, nil)
    wait_for(function()
      return #fake.of("push") == 1
    end)
    assert(vim.deep_equal(fake.of("push")[1].argv, { "dam", "push", "-o", "json" }))
  end,

  ["a spy on the Progress autocmd sees one message running then success"] = function()
    local app, fake = helper.new()
    local seen = spied()
    replied(fake, fixtures.read("progress-push.txt"))
    local got = push(app, true)
    wait_for(function()
      return got.done
    end)
    assert(vim.deep_equal(statuses(seen), { "running", "success" }), vim.inspect(seen))
    assert(vim.deep_equal(texts(seen), { "push", "push finished" }), vim.inspect(seen))
    assert(seen[1].id == seen[2].id and seen[1].text[1] == "push" and seen[1].title == "fake", vim.inspect(seen))
    assert(got.answer.remotes[1].remote == "fake", vim.inspect(got))
  end,

  ["a failed step ends its message as failed and the call gives dam's error"] = function()
    local app, fake = helper.new()
    local seen = spied()
    replied(fake, fixtures.read("progress-push-failed.txt"), 1)
    local got = push(app, true)
    wait_for(function()
      return got.done
    end)
    assert(vim.deep_equal(statuses(seen), { "running", "failed" }), vim.inspect(seen))
    assert(got.err.kind == "helper", vim.inspect(got.err))
    assert(vim.deep_equal(texts(seen), { "push", "push failed" }), vim.inspect(seen))
  end,

  ["a step still running when dam exits ends as failed"] = function()
    local app, fake = helper.new()
    local seen = spied()
    replied(fake, vim.split(fixtures.read("progress-push.txt"), "\n")[1] .. "\ndam: killed\n", 1)
    local got = push(app, true)
    wait_for(function()
      return got.done
    end)
    assert(vim.deep_equal(statuses(seen), { "running", "failed" }), vim.inspect(seen))
    assert(got.err.message == "dam: killed", vim.inspect(got.err))
    assert(vim.deep_equal(texts(seen), { "push", "push failed" }), vim.inspect(seen))
  end,

  ["a cancelled call ends its progress message as cancelled"] = function()
    local app, fake = helper.new()
    fake.app = app
    local seen = spied()
    local got = started(fake, seen)
    got.handle.cancel()
    vim.wait(30)
    assert(vim.deep_equal(statuses(seen), { "running", "cancel" }), vim.inspect(seen))
    assert(vim.deep_equal(texts(seen), { "push", "push cancelled" }), vim.inspect(seen))
    assert(fake.of("push")[1].killed and not got.done, vim.inspect(got))
  end,

  ["cancelling the progress message cancels its call"] = function()
    local app, fake = helper.new()
    fake.app = app
    local seen = spied()
    local got = started(fake, seen)
    vim.api.nvim_echo({ { "push" } }, false, { id = seen[1].id, kind = "progress", status = "cancel", source = "dam" })
    vim.wait(30)
    assert(fake.of("push")[1].killed and not got.done, vim.inspect(fake.of("push")))
    assert(vim.deep_equal(statuses(seen), { "running", "cancel" }), vim.inspect(seen))
  end,
}
