local buffer = require("helpers.buffer")
local log = require("helpers.log")
local real_dam = require("helpers.real_dam")
local wait_for = buffer.wait_for

local function committed(app, sandbox, write, message)
  local made = real_dam.made(app, write, sandbox)
  real_dam.made(app, { "add", "-A" }, sandbox)
  real_dam.made(app, { "commit", "-m", message }, sandbox)
  return made
end

local function planted(sandbox)
  require("damnit").setup({ dam = real_dam.bin })
  local app = real_dam.app()
  local oid = committed(app, sandbox, { "new", "plant the tomatoes" }, "plant")
  vim.cmd.lcd(sandbox)
  return app, oid
end

return {
  ["= on a change in the status window shows its fields before → after"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local app, oid = planted(sandbox)
      real_dam.made(app, { "edit", oid, "--priority", "1", "--labels", "garden" }, sandbox)
      vim.cmd("Dam status")
      local row = "  update  plant the tomatoes"
      wait_for(function()
        return vim.tbl_contains(buffer.lines(), row)
      end, 20000)
      vim.api.nvim_win_set_cursor(0, { vim.fn.index(buffer.lines(), row) + 1, 0 })
      buffer.feed("=")
      local shown = log.virtual_lines()[vim.fn.index(buffer.lines(), row) + 1]
      assert(vim.deep_equal(shown, { "    labels    none → garden", "    priority  4 → 1" }), vim.inspect(shown))
    end)
  end,

  ["R in the log brings a committed change back as a working change"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local app, oid = planted(sandbox)
      committed(app, sandbox, { "edit", oid, "--priority", "1" }, "raise it")
      vim.cmd("Dam log")
      wait_for(function()
        return vim.bo.filetype == "damlog" and (buffer.lines()[1] or ""):find("raise it", 1, true) ~= nil
      end, 20000)
      vim.api.nvim_win_set_cursor(0, { 1, 0 })
      buffer.feed("R")
      wait_for(function()
        return vim.bo.filetype == "damconfirm"
      end, 20000)
      buffer.feed("y")
      wait_for(function()
        return vim.bo.filetype == "damstatus" and vim.tbl_contains(buffer.lines(), "  update  plant the tomatoes")
      end, 20000)
      assert(buffer.lines()[1] == "Unstaged (1)", vim.inspect(buffer.lines()))
    end)
  end,
}
