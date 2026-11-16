local buffer = require("helpers.buffer")
local helper = require("helpers.app")
local prompt = require("helpers.prompt")
local real_dam = require("helpers.real_dam")
local ui = require("helpers.ui")
local wait_for = buffer.wait_for

local function listed(sandbox)
  local app = real_dam.app()
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  real_dam.made(app, { "new", "fix the sink", "--labels", "bug" }, sandbox)
  real_dam.made(app, { "new", "buy oat milk", "--labels", "errand" }, sandbox)
  vim.cmd.lcd(sandbox)
  vim.cmd("Dam list")
  wait_for(function()
    return #buffer.lines() == 2
  end, 20000)
  return app, vim.api.nvim_get_current_buf()
end

return {
  ["typing --labels and a label in the filter bar narrows the list to dam's answer"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local _, list_buf = listed(sandbox)
      prompt.type("f--labels bug")
      prompt.footer_reads("1 row", 20000)
      assert(vim.deep_equal(buffer.lines(list_buf), { "[ ] fix the sink" }), vim.inspect(buffer.lines(list_buf)))
    end)
  end,

  ["<C-s> saves the bar's flags as a filter that dam filter list then names"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local app = listed(sandbox)
      local notes = buffer.capture_notes()
      prompt.type("f--labels bug")
      prompt.footer_reads("1 row", 20000)
      ui.answering({ input = "bugs" }, function()
        prompt.more("<C-s>")
        wait_for(function()
          return #notes > 0
        end, 20000)
      end)
      assert(notes[1].text == "saved filter bugs", vim.inspect(notes))
      local filters = helper.call(app, { args = { "filter", "list" }, cwd = sandbox })
      wait_for(function()
        return filters.done
      end, 20000)
      local names = vim.tbl_map(function(filter)
        return filter.name
      end, filters.answer.filters)
      assert(vim.tbl_contains(names, "bugs"), vim.inspect(filters.answer))
      wait_for(function()
        return vim.api.nvim_buf_get_name(0) == "dam://list/bugs" and #buffer.lines() == 1
      end, 20000)
    end)
  end,
}
