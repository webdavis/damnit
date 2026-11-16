local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")

local UNSTAGED = "  update  call mom"
local STAGED = "  update  buy oat milk"

local function opened()
  local app, fake = status.defined()
  fake.reply("status", fixtures.answer("status-full.json"))
  status.open(fake)
  return app, fake
end

---@return table<integer, string[]> each line's virtual lines as text, keyed by line from 1
local function shown()
  local namespace = vim.api.nvim_create_namespace("damnit")
  local found = {}
  for _, mark in ipairs(vim.api.nvim_buf_get_extmarks(0, namespace, 0, -1, { details = true })) do
    if mark[4].virt_lines then
      found[mark[2] + 1] = vim.tbl_map(function(chunks)
        return table.concat(vim.tbl_map(function(chunk)
          return chunk[1]
        end, chunks))
      end, mark[4].virt_lines)
    end
  end
  return found
end

local function row_of(text)
  return vim.fn.index(buffer.lines(), text) + 1
end

return {
  ["= on a change shows each field before and after under it, and = again hides them"] = function()
    opened()
    local lines = #buffer.lines()
    status.on(UNSTAGED)
    buffer.feed("=")
    local expected = { "    labels    none → phone", "    priority  4 → 1", "    due       none → 2026-09-30" }
    assert(vim.deep_equal(shown(), { [row_of(UNSTAGED)] = expected }), vim.inspect(shown()))
    assert(#buffer.lines() == lines and vim.api.nvim_get_current_line() == UNSTAGED)
    buffer.feed("=")
    assert(vim.tbl_isempty(shown()), vim.inspect(shown()))
  end,

  ["= on a section header shows every row of that section"] = function()
    opened()
    status.on("Waiting (2)")
    buffer.feed("=")
    local open = shown()
    assert(vim.deep_equal(open[row_of("  new       call the plumber")], { "    subject  none → call the plumber" }))
    local set = { "    call mom      priority  1 → 3", "    buy oat milk  priority  4 → 3" }
    assert(vim.deep_equal(open[row_of("  set of 2")], set), vim.inspect(open))
    buffer.feed("=")
    assert(vim.tbl_isempty(shown()), vim.inspect(shown()))
  end,

  ["what = opened stays open when the window reads again"] = function()
    local _, fake = opened()
    status.on(STAGED)
    buffer.feed("=")
    vim.api.nvim_exec_autocmds("User", { pattern = "DamChanged", data = {} })
    buffer.wait_for(function()
      return #fake.of("status") == 2
    end)
    vim.wait(20)
    assert(shown()[row_of(STAGED)] ~= nil, vim.inspect(shown()))
  end,

  ["= on a row with no fields warns"] = function()
    opened()
    local notes = buffer.capture_notes()
    status.on("  fake  1 commit")
    buffer.feed("=")
    assert(#notes == 1 and notes[1].level == vim.log.levels.WARN, vim.inspect(notes))
  end,

  ["= on a removed event shows one line saying dam removes it"] = function()
    opened()
    status.on("  delete  standup")
    buffer.feed("=")
    local open = shown()[row_of("  delete  standup")]
    assert(vim.deep_equal(open, { "    delete: the event is removed" }), vim.inspect(shown()))
  end,

  ["= on a change with nothing to show says so and opens nothing"] = function()
    local _, fake = status.defined()
    local answer = fixtures.decoded("status-full.json")
    answer.unstaged[2].op = "update"
    fake.reply("status", { code = 0, stdout = vim.json.encode(answer), stderr = "" })
    status.open(fake)
    local notes = buffer.capture_notes()
    status.on("  update  standup")
    buffer.feed("=")
    assert(#notes == 1 and notes[1].text == "nothing more to show for this row", vim.inspect(notes))
    assert(vim.tbl_isempty(shown()), vim.inspect(shown()))
  end,
}
