local look = require("damnit.list.look")
local winbar = require("damnit.list.winbar")

local Drawing = {}

---@param buf integer
function Drawing:draw(buf)
  local list = self.shown[buf]
  list.width = self.app.screen:text_width(buf)
  self.app.screen:draw(buf, look.render(list.arranged, self.app.options.signs, list.width))
  local objects = vim.tbl_map(function(row)
    return row.object
  end, list.arranged)
  local bar = winbar.chunks(list.name, list.path or vim.fs.basename(list.call.cwd), objects)
  self.app.screen:window(buf, { winbar = self.app.screen.bar(bar), signcolumn = "auto:2", wrap = false })
end

---@param buf integer
function Drawing:refit(buf)
  local list = self.shown[buf]
  if list and list.arranged and vim.api.nvim_buf_is_valid(buf) and self.app.screen:text_width(buf) ~= list.width then
    self:draw(buf)
  end
end

return Drawing
