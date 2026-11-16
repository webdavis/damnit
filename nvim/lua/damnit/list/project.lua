local M = {}

---@class damnit.ListProject
---@field path fun(cwd: string, done: fun(path: string))

---@param app damnit.App
---@return damnit.ListProject
function M.new(app)
  local known = {}
  return {
    path = function(cwd, done)
      if known[cwd] then
        return done(known[cwd])
      end
      app.dam:run({ args = { "config", "list" }, cwd = cwd }, function(answer)
        known[cwd] = answer and answer.project and answer.project.path or vim.fs.basename(cwd)
        done(known[cwd])
      end)
    end,
  }
end

return M
