local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local prompt = require("helpers.prompt")
local real_dam = require("helpers.real_dam")
local wait_for = buffer.wait_for

local function listed(sandbox, subjects)
  local sent = {}
  local process = require("damnit.kernel.process")
  local port = {
    run = function(argv, opts, on_exit)
      table.insert(sent, argv)
      return process.run(argv, opts, on_exit)
    end,
  }
  local options = require("damnit.kernel.options").resolve({ dam = real_dam.bin })
  local app = require("damnit.kernel.app").new(options, port)
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  for _, subject in ipairs(subjects) do
    real_dam.made(app, { "new", subject }, sandbox)
  end
  vim.cmd.lcd(sandbox)
  vim.cmd("Dam list")
  wait_for(function()
    return #buffer.lines() == #subjects and buffer.lines()[1] ~= ""
  end, 20000)
  return sent
end

local function writes(sent, verb)
  return vim.tbl_filter(function(argv)
    return argv[2] == verb and not vim.tbl_contains(argv, "--preview")
  end, sent)
end

local function confirmed()
  wait_for(function()
    return vim.bo.filetype == "damconfirm"
  end, 20000)
  buffer.feed("y")
end

return {
  ["p2 on three marked rows changes all three in one call"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local sent = listed(sandbox, { "sow beans", "water seedlings", "stake tomatoes" })
      vim.api.nvim_win_set_cursor(0, { 1, 0 })
      buffer.feed("3m")
      buffer.feed("p2")
      confirmed()
      wait_for(function()
        return #vim.tbl_filter(function(line)
          return line:find("^%[ %] p2 ") ~= nil
        end, buffer.lines()) == 3
      end, 20000)
      assert(#writes(sent, "edit") == 1, vim.inspect(writes(sent, "edit")))
    end)
  end,

  ["Vjx completes two rows in one call"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local sent = listed(sandbox, { "sow beans", "water seedlings", "stake tomatoes" })
      vim.api.nvim_win_set_cursor(0, { 1, 0 })
      buffer.feed("Vjx")
      confirmed()
      wait_for(function()
        return #buffer.lines() == 1
      end, 20000)
      assert(#writes(sent, "done") == 1, vim.inspect(writes(sent, "done")))
    end)
  end,

  ["cd with a date dam cannot read shows dam's message and sends nothing"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local sent = listed(sandbox, { "sow beans" })
      prompt.type([[cd"the twelfth of never"]])
      prompt.below_reads(fixtures.decoded("error-edit-due.json").error.message, 20000)
      prompt.more("<CR>")
      vim.wait(200)
      assert(#writes(sent, "edit") == 0 and prompt.window(), vim.inspect(writes(sent, "edit")))
    end)
  end,

  ["cd takes next fri unquoted and shows the date dam reads"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local sent = listed(sandbox, { "sow beans" })
      local read = require("helpers.app").call(
        real_dam.app(),
        { args = { "new", "x", "--due", "next fri", "--preview" }, cwd = sandbox }
      )
      wait_for(function()
        return read.done
      end, 20000)
      prompt.type("cdnext fri")
      prompt.footer_reads("due  none → " .. read.answer.changes[1].due, 20000)
      prompt.more("<CR>")
      wait_for(function()
        return #writes(sent, "edit") == 1
      end, 20000)
      local edit = writes(sent, "edit")[1]
      assert(vim.deep_equal(vim.list_slice(edit, 4, 5), { "--due", "next fri" }), vim.inspect(edit))
    end)
  end,
}
