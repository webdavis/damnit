local Judge = {}
Judge.__index = Judge

local M = {}

local PAUSE_MS = 120

---@class damnit.Judge
---@field soon fun(self, text: string)
---@field now fun(self, text: string, after: fun(refused: boolean))
---@field stop fun(self)

---@param queue damnit.Queue
---@param lane string
---@param read fun(text: string, done: fun(answer: string, refused: boolean)): damnit.Handle
---@param show fun(answer: string, refused: boolean)
---@return damnit.Judge
function M.new(queue, lane, read, show)
  return setmetatable(
    { queue = queue, lane = lane, read = read, show = show, timer = assert(vim.uv.new_timer()) },
    Judge
  )
end

function Judge:run(text, after)
  self.timer:stop()
  self.waiting, self.asked = nil, text
  self.queue:submit(self.lane, function(finish)
    return self.read(text, function(answer, refused)
      finish()
      if self.asked ~= text then
        return
      end
      self.judged = { text = text, refused = refused }
      self.show(answer, refused)
      if after then
        after(refused)
      end
    end)
  end, "latest")
end

function Judge:soon(text)
  if text == self.waiting or text == self.asked or (self.judged and self.judged.text == text) then
    return
  end
  self:stop()
  self.waiting = text
  self.timer:start(
    PAUSE_MS,
    0,
    vim.schedule_wrap(function()
      if self.waiting == text then
        self:run(text)
      end
    end)
  )
end

function Judge:now(text, after)
  if self.judged and self.judged.text == text then
    return after(self.judged.refused)
  end
  self:run(text, after)
end

function Judge:stop()
  self.waiting, self.asked, self.judged = nil, nil, nil
  self.queue:cancel(self.lane)
  self.timer:stop()
end

function Judge:close()
  self:stop()
  self.timer:close()
end

return M
