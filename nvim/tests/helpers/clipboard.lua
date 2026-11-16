local M = {}

---@return table<string, string[]> held what each register would hold in the system clipboard
function M.fake()
  local held = {}
  local function copy(register)
    return function(lines)
      held[register] = lines
    end
  end
  local function paste(register)
    return function()
      return held[register] or {}
    end
  end
  vim.g.clipboard = {
    name = "spec",
    copy = { ["+"] = copy("+"), ["*"] = copy("*") },
    paste = { ["+"] = paste("+"), ["*"] = paste("*") },
  }
  vim.g.loaded_clipboard_provider = nil
  vim.cmd("runtime autoload/provider/clipboard.vim")
  return held
end

return M
