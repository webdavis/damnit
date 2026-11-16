local prompt = require("helpers.prompt")
local wait_for = require("helpers.buffer").wait_for

local FLAGS = { { name = "--due", value = "required", help = "Set the due date" } }

local function opened(keys, below)
  local prompts =
    require("damnit.kernel.prompt").new(require("damnit.kernel.queue").new(), require("damnit.kernel.say").new())
  local reads = {}
  prompt.open(function()
    prompts:open({
      title = "dam edit",
      prefill = "",
      flags = FLAGS,
      read = function(text, done)
        table.insert(reads, text)
        done("3 of 3 would change", false, below)
        return { cancel = function() end }
      end,
      submit = function() end,
    })
  end, keys)
  return reads
end

return {
  ["a read's lines show under the input with its answer in the footer"] = function()
    local rows = { "call mom  due  none → 2026-11-20", "pay rent  due  2026-09-30 → 2026-11-20" }
    opened("--due fri", rows)
    prompt.footer_reads("3 of 3 would change")
    assert(vim.deep_equal(prompt.below(), rows), vim.inspect(prompt.below()))
    assert(vim.api.nvim_win_get_height(assert(prompt.window())) == 3)
  end,

  ["while the word at the cursor is an unfinished flag name the prompt reads nothing and shows nothing"] = function()
    local reads = opened("--d")
    vim.wait(250)
    assert(#reads == 0 and prompt.footer() == "", vim.inspect(reads))
    prompt.more("ue fri")
    prompt.footer_reads("3 of 3 would change")
    assert(vim.deep_equal(reads, { "--due fri" }), vim.inspect(reads))
  end,

  ["a word that is a whole flag name is read"] = function()
    local reads = opened("--due")
    wait_for(function()
      return #reads == 1
    end)
  end,
}
