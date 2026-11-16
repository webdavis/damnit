local Queue = {}
Queue.__index = Queue

local M = {}

---@return damnit.Queue
function M.new()
  return setmetatable({ lanes = {} }, Queue)
end

function Queue:advance(name)
  local lane = self.lanes[name]
  if not lane or lane.running then
    return
  end
  local start = table.remove(lane.pending, 1)
  if not start then
    self.lanes[name] = nil
    return
  end
  local entry = {}
  lane.running = entry
  entry.handle = start(function()
    if lane.running == entry then
      lane.running = nil
      self:advance(name)
    end
  end)
end

---@param name string
---@param start fun(finish: fun()): damnit.Handle
---@param mode "latest"|"ordered"
function Queue:submit(name, start, mode)
  local lane = self.lanes[name] or { pending = {} }
  self.lanes[name] = lane
  if mode == "latest" then
    self:stop(lane)
  end
  table.insert(lane.pending, start)
  self:advance(name)
end

function Queue:stop(lane)
  lane.pending = {}
  local running = lane.running
  lane.running = nil
  if running and running.handle then
    running.handle.cancel()
  end
end

---@param name? string
---@return integer stopped
function Queue:cancel(name)
  local stopped = 0
  for lane_name, lane in pairs(self.lanes) do
    if name == nil or lane_name == name then
      stopped = stopped + #lane.pending + (lane.running and 1 or 0)
      self:stop(lane)
      self.lanes[lane_name] = nil
    end
  end
  return stopped
end

return M
