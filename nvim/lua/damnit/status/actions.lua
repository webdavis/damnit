local M = {}

local function on_row(verb)
  return function(target)
    return { verb, target.id }
  end
end

local function always(...)
  local argv = { ... }
  return function()
    return argv
  end
end

local KEYS = {
  s = {
    fits = "s stages an unstaged change, or everything on the Unstaged or Waiting header",
    rows = {
      ["change:unstaged"] = on_row("add"),
      ["section:unstaged"] = always("add", "-A"),
      ["section:waiting"] = always("add", "-A"),
    },
  },
  u = {
    fits = "u unstages a staged change, or everything on the Staged header",
    rows = { ["change:staged"] = on_row("reset"), ["section:staged"] = always("reset") },
  },
  a = {
    fits = "a accepts a waiting item or set",
    rows = { ["waiting:waiting"] = on_row("add"), ["set:waiting"] = on_row("add") },
  },
  dd = {
    fits = "dd declines a waiting item or set",
    rows = { ["waiting:waiting"] = on_row("rm"), ["set:waiting"] = on_row("rm") },
  },
  U = {
    fits = "U sets an unstaged or staged change back to its last commit",
    rows = { ["change:unstaged"] = on_row("restore"), ["change:staged"] = on_row("restore") },
  },
}

---@alias damnit.StatusKey "s"|"u"|"a"|"dd"|"U"

---@param target damnit.StatusTarget|nil
---@param key damnit.StatusKey
---@return string[]|nil
function M.argv(target, key)
  local build = target and KEYS[key].rows[target.kind .. ":" .. target.section]
  return build and build(target) or nil
end

---@param key damnit.StatusKey
---@return string
function M.fits(key)
  return KEYS[key].fits
end

---@param doc damnit.StatusDoc
---@param target damnit.StatusTarget
---@return damnit.ConfirmRow[] rows, string summary
function M.declined(doc, target)
  for _, item in ipairs(doc.waiting) do
    if item.hash == target.id then
      return { { op = item.item, subject = item.subject, detail = "from " .. item.from } },
        "1 waiting item would be declined"
    end
  end
  for _, set in ipairs(doc.sets) do
    if set.set == target.id then
      local summary = ("a set of %d would be declined"):format(set.count)
      return { { op = ("set of %d"):format(set.count), subject = "", detail = "from " .. set.from } }, summary
    end
  end
  return {}, "nothing would be declined"
end

return M
