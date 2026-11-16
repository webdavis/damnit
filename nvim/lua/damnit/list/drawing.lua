local folds = require("damnit.kernel.folds")
local groups = require("damnit.list.groups")
local look = require("damnit.list.look")
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
  local bar = winbar.chunks(list.name, list.path or vim.fs.basename(list.call.cwd), objects)
  local window = { winbar = self.app.screen.bar(bar), signcolumn = "auto:2", wrap = false }
  local by_path = groups.key(list.grouped_by) == "path"
  folds.redraw(buf, fold_lines, function()
    self.app.screen:window(buf, vim.tbl_extend("error", window, folds.window))
    self.app.screen:draw(buf, look.render(list.arranged, self.app.options.signs, list.width, by_path))
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
