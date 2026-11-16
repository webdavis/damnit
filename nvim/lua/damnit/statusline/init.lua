local format = require("damnit.statusline.format")

local M = {}

local LANE = "statusline"

---@class damnit.Statusline
---@field component fun(opts?: { highlight?: boolean }): string
---@field counts fun(): damnit.Counts|nil

---@param app damnit.App
---@return damnit.Statusline
function M.new(app)
  local state = { reads = 0 }

  local function read()
    state.reads = state.reads + 1
    local this, cwd = state.reads, vim.fn.getcwd()
    local function current(answer)
      return answer ~= nil and this == state.reads
    end
    app.dam:run({ args = { "list", "--overdue" }, cwd = cwd, key = LANE }, function(overdue)
      if not current(overdue) then
        return
      end
      app.dam:run({ args = { "status" }, cwd = cwd, key = LANE }, function(status)
        if current(status) then
          state.counts = format.counts(overdue, status)
          vim.cmd.redrawstatus({ bang = true })
        end
      end)
    end)
  end

  local function started()
    if state.started then
      return
    end
    state.started = true
    local group = vim.api.nvim_create_augroup("damnit.statusline", { clear = true })
    vim.api.nvim_create_autocmd("User", { group = group, pattern = "DamChanged", callback = read })
    vim.api.nvim_create_autocmd({ "FocusGained", "DirChanged" }, { group = group, callback = read })
    vim.schedule(read)
  end

  return {
    component = function(opts)
      started()
      return format.text(state.counts, app.options.signs, opts and opts.highlight)
    end,
    counts = function()
      started()
      return state.counts and vim.deepcopy(state.counts)
    end,
  }
end

return M
