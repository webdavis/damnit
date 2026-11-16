local folds = require("damnit.kernel.folds")
local groups = require("damnit.list.groups")
local look = require("damnit.list.look")
local marked = require("damnit.list.marked")
local winbar = require("damnit.list.winbar")

local Drawing = {}

---@param buf integer
function Drawing:draw(buf)
  local list = self.shown[buf]
  list.width = self.app.screen:text_width(buf)
  local objects, fold_lines = {}, {}
  for n, row in ipairs(list.arranged) do
    table.insert(objects, row.object)
    local object = row.object or {}
    fold_lines[n] =
      { header = row.header ~= nil, depth = row.depth, open = object.children_open, key = row.key or object.oid }
  end
  list.marked = marked.kept(list.marked or {}, list.oids)
  local bar = winbar.chunks(list.name, list.path or vim.fs.basename(list.call.cwd), objects, marked.count(list.marked))
  local window = { winbar = self.app.screen.bar(bar), signcolumn = "auto:2", wrap = false }
  local by_path = groups.key(list.grouped_by) == "path"
  folds.redraw(buf, fold_lines, function()
    self.app.screen:window(buf, vim.tbl_extend("error", window, folds.window))
    local lines, marks = look.render(list.arranged, self.app.options.signs, list.width, by_path)
    self.app.screen:draw(buf, lines, vim.list_extend(marks, marked.marks(list.marked, list.oids)))
  end)
end

---@param buf integer
function Drawing:refit(buf)
  local list = self.shown[buf]
  if list and list.arranged and vim.api.nvim_buf_is_valid(buf) and self.app.screen:text_width(buf) ~= list.width then
    self:draw(buf)
  end
end

return Drawing
