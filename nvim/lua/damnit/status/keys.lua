local actions = require("damnit.status.actions")
local folds = require("damnit.kernel.folds")

local Keys = {}

local GROUP = "status"

---@param key damnit.StatusKey
function Keys:act(key)
  local target = self:target()
  local argv = actions.argv(target, key)
  if not argv then
    return self.app.say:warn(actions.fits(key))
  end
  if key == "dd" then
    return self:decline(target, argv)
  end
  if key == "U" then
    return self:restore(argv)
  end
  self:send(argv)
end

---@param buf integer
---@return damnit.KeySpec[]
function Keys:task_keys(buf)
  return self.app.keys:for_task(buf, GROUP, function()
    return (self:target() or {}).oid, self.cwd
  end)
end

---@param buf integer
function Keys:bind(buf)
  local function call(lhs, desc, run)
    return { lhs = lhs, desc = desc, group = GROUP, run = run }
  end
  local function act(key, desc)
    return call(key, desc, function()
      self:act(key)
    end)
  end
  self.app.keys:bind(
    buf,
    vim.list_extend({
      act("s", "stage the change, or everything on a header"),
      call("S", "stage everything", function()
        self:send({ "add", "-A" })
      end),
      act("u", "unstage the change, or everything on the Staged header"),
      act("a", "accept the waiting item or set"),
      act("dd", "decline the waiting item or set, after the confirm float"),
      act("U", "set the change back to its last commit, after the confirm float"),
      call("=", "show or hide what the row changes, or every row's on a header", function()
        self:toggle_details()
      end),
      call("cc", "commit the stage", function()
        self.writes.commit(self.cwd)
      end),
      call("P", "push to every remote, after the confirm float", function()
        self:push()
      end),
      call("F", "pull from every remote", function()
        self.writes.pull(self.cwd)
      end),
      call("<CR>", "open the task", function()
        self:open_target_task()
      end),
      call("r", "read the status again", function()
        self:read()
      end),
    }, vim.list_extend(folds.keys(), self:task_keys(buf)))
  )
  vim.api.nvim_create_autocmd("BufWipeout", {
    group = self.group,
    buffer = buf,
    callback = function()
      self.buf, self.answer = nil, nil
    end,
  })
end

return Keys
