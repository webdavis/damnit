local fake_process = require("helpers.fake_process")

local M = {}

---@param user_options? table
---@return damnit.App app, table fake
function M.new(user_options)
  local fake = fake_process.new()
  local options = require("damnit.kernel.options").resolve(user_options or {})
  return require("damnit.kernel.app").new(options, fake.port), fake
end

---@param app damnit.App
---@param call damnit.Call
---@param method? "run"|"write"
function M.call(app, call, method)
  local got = { done = false }
  got.handle = app.dam[method or "run"](app.dam, call, function(answer, err)
    got.done, got.answer, got.err = true, answer, err
  end)
  return got
end

return M
