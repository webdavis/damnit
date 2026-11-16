local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")

local M = {}

---@return damnit.App app, table fake
function M.defined()
  local app, fake = status.defined()
  local log, window
  log = require("damnit.log").new(app, function(cwd)
    window.open(cwd)
  end)
  window = require("damnit.status").new(app, function() end, log.open)
  require("damnit.command").define(app, { log, window })
  fake.reply("show", fixtures.answer("show-commit.json"))
  fake.reply("revert", fixtures.answer("revert.json"))
  return app, fake
end

---@param fake table
function M.open(fake)
  local before = #fake.of("log")
  vim.cmd("Dam log")
  buffer.wait_for(function()
    return #fake.of("log") > before and vim.bo.filetype == "damlog" and #buffer.lines() == 3
  end)
end

---@return damnit.App app, table fake
function M.opened()
  local app, fake = M.defined()
  M.open(fake)
  return app, fake
end

---@return table<integer, string[]> each line's virtual lines as text, keyed by line from 1
function M.virtual_lines()
  local found = {}
  local namespace = vim.api.nvim_create_namespace("damnit")
  for _, mark in ipairs(vim.api.nvim_buf_get_extmarks(0, namespace, 0, -1, { details = true })) do
    if mark[4].virt_lines then
      found[mark[2] + 1] = vim.tbl_map(function(chunks)
        return table.concat(vim.tbl_map(function(chunk)
          return chunk[1]
        end, chunks))
      end, mark[4].virt_lines)
    end
  end
  return found
end

return M
