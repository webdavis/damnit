local buffer = require("helpers.buffer")
local real_dam = require("helpers.real_dam")

local function card_lines()
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_config(win).relative ~= "" then
      return vim.api.nvim_buf_get_lines(vim.api.nvim_win_get_buf(win), 0, -1, false)
    end
  end
end

return {
  ["K on a list row shows that task's body in a float"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local app = real_dam.app()
      require("damnit.peek").new(app, function() end)
      require("damnit.command").define(app, { require("damnit.list").new(app, function() end) })
      real_dam.made(app, { "new", "plant the tomatoes", "--body", "loosen the soil\nwater after" }, sandbox)
      vim.cmd.lcd(sandbox)
      vim.cmd("Dam list")
      buffer.wait_for(function()
        return #buffer.lines() == 1 and buffer.lines()[1]:find("plant the tomatoes", 1, true) ~= nil
      end, 20000)
      buffer.feed("K")
      buffer.wait_for(function()
        return card_lines() ~= nil
      end, 20000)
      local lines = card_lines()
      assert(
        vim.deep_equal(vim.list_slice(lines, #lines - 1), { "loosen the soil", "water after" }),
        vim.inspect(lines)
      )
    end)
  end,
}
