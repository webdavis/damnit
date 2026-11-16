local Vocab = {}
Vocab.__index = Vocab

local M = {}

local function each(list, make)
  return vim.tbl_map(make, list or {})
end

local SOURCES = {
  filters = {
    args = { "filter", "list" },
    read = function(answer)
      return each(answer.filters, function(filter)
        return { word = filter.name, menu = "saved filter" }
      end)
    end,
  },
  categories = {
    args = { "category", "list" },
    read = function(answer)
      return each(answer.categories, function(category)
        return { name = category.name, values = category.values }
      end)
    end,
  },
  remotes = {
    args = { "remote", "list" },
    read = function(answer)
      return each(answer.remotes, function(remote)
        return { word = remote.name, menu = "remote" }
      end)
    end,
  },
  labels = {
    args = { "label", "list" },
    read = function(answer)
      return each(answer.labels, function(label)
        return { word = label.name, menu = ("label · %d"):format(label.count) }
      end)
    end,
  },
  paths = {
    args = { "path", "list" },
    read = function(answer)
      return each(answer.paths, function(path)
        return { word = path.path, menu = ("path · %d"):format(path.count) }
      end)
    end,
  },
}

local FLAGS = vim.fs.joinpath(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)), "flags.json")

---@param dam damnit.Dam
---@param say damnit.Say
---@return damnit.VocabCache
function M.new(dam, say)
  local vocab = setmetatable({ dam = dam, say = say, folders = {} }, Vocab)
  vim.api.nvim_create_autocmd("User", {
    group = vim.api.nvim_create_augroup("damnit.vocab", { clear = true }),
    pattern = "DamChanged",
    callback = function()
      vocab.folders = {}
    end,
  })
  return vocab
end

local function unknown_command(err)
  return type(err.usage) == "table" and err.usage.problem == "unknown_subcommand"
end

function Vocab:read(cwd, words)
  for key, source in pairs(SOURCES) do
    local call = { args = source.args, cwd = cwd, key = ("vocab %s %s"):format(key, cwd) }
    self.dam:run(call, function(answer, err)
      if err and not unknown_command(err) then
        self.say:error(err)
      end
      words[key] = answer and source.read(answer) or {}
    end)
  end
end

---@param cwd string
---@return damnit.Vocab
function Vocab:words(cwd)
  if not self.folders[cwd] then
    self.folders[cwd] = {}
    self:read(cwd, self.folders[cwd])
  end
  return self.folders[cwd]
end

---@param command string
---@return damnit.Flag[]
function Vocab:flags(command)
  if not self.every_flag then
    local file = assert(io.open(FLAGS, "r"))
    self.every_flag = vim.json.decode(file:read("*a"))
    file:close()
  end
  return self.every_flag[command] or {}
end

return M
