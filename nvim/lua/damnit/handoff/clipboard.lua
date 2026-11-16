local M = {}

---@return damnit.Host
function M.new()
  return {
    name = "clipboard",
    available = function()
      return true
    end,
    send = function(brief, done)
      vim.fn.setreg('"', brief)
      if vim.fn.has("clipboard") == 0 then
        return done(nil, "brief copied to the unnamed register; Neovim has no clipboard tool")
      end
      vim.fn.setreg("+", brief)
      done(nil, "brief copied to the clipboard")
    end,
  }
end

return M
