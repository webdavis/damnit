local buffer = require("helpers.buffer")
local helper = require("helpers.app")
local wait_for = buffer.wait_for

local M = {}

local tests_dir = vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)))
M.bin = vim.fs.normalize(vim.fs.abspath(vim.env.DAM_BIN or vim.fs.joinpath(tests_dir, "../../target/debug/dam")))

local SANDBOXED = { "DAM_CONFIG", "DAM_STORE", "HOME" }

local function in_sandbox(run)
  local sandbox = vim.fn.tempname()
  vim.fn.mkdir(sandbox, "p")
  local saved = {}
  for _, name in ipairs(SANDBOXED) do
    saved[name] = vim.env[name]
  end
  vim.env.DAM_CONFIG = vim.fs.joinpath(sandbox, "config", "dam", "config.lua")
  vim.env.DAM_STORE = vim.fs.joinpath(sandbox, "dam.db")
  vim.env.HOME = sandbox
  local ok, err = pcall(run, sandbox)
  for _, name in ipairs(SANDBOXED) do
    vim.env[name] = saved[name]
  end
  assert(ok, err)
end

---@param run fun(sandbox: string)
---@return string|nil skipped
function M.with_real_dam(run)
  if vim.fn.executable(M.bin) ~= 1 then
    return ("no dam at %s; build it with cargo build -p damnit or set DAM_BIN"):format(M.bin)
  end
  in_sandbox(run)
end

---@return damnit.App
function M.app()
  local options = require("damnit.kernel.options").resolve({ dam = M.bin })
  return require("damnit.kernel.app").new(options, require("damnit.kernel.process"))
end

---@param app damnit.App
---@param args string[]
---@param cwd string
---@return string oid
function M.made(app, args, cwd)
  local got = helper.call(app, { args = args, cwd = cwd }, "write")
  wait_for(function()
    return got.done
  end, 20000)
  assert(not got.err, vim.inspect(got.err))
  return got.answer.oid
end

return M
