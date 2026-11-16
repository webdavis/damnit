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
  x = {
    fits = "x declines a waiting item or set",
    rows = { ["waiting:waiting"] = on_row("rm"), ["set:waiting"] = on_row("rm") },
  },
}

---@param target damnit.StatusTarget|nil
---@param key "s"|"u"|"a"|"x"
---@return string[]|nil
function M.argv(target, key)
  local build = target and KEYS[key].rows[target.kind .. ":" .. target.section]
  return build and build(target) or nil
end

---@param key "s"|"u"|"a"|"x"
---@return string
function M.fits(key)
  return KEYS[key].fits
end

return M
