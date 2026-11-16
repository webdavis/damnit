local prompt = require("helpers.prompt")
local wait_for = require("helpers.buffer").wait_for

local function opened(spec, keys)
  local prompts = require("damnit.kernel.prompt").new(require("damnit.kernel.queue").new())
  local seen = { reads = {}, submitted = {}, cancelled = 0 }
  local full = vim.tbl_extend("keep", spec or {}, {
    title = "dam new",
    prefill = "",
    read = function(text, done)
      table.insert(seen.reads, text)
      local timer = vim.defer_fn(function()
        done(text:find("never") and "cannot read never" or ("read " .. text), text:find("never") ~= nil)
      end, text == "buy" and 5000 or 10)
      return {
        cancel = function()
          seen.cancelled = seen.cancelled + 1
          timer:stop()
        end,
      }
    end,
    submit = function(text)
      table.insert(seen.submitted, text)
    end,
  })
  prompt.open(function()
    prompts:open(full)
  end, keys)
  return seen, prompts
end

return {
  ["the prompt is a one-line float titled by the caller, in insert mode after the prefill"] = function()
    opened({ prefill = "buy ", after = " --path home/" }, "milk")
    local win = assert(prompt.window())
    local config = vim.api.nvim_win_get_config(win)
    assert(config.relative == "editor" and config.height == 1, vim.inspect(config))
    assert(vim.trim(config.title[1][1]) == "dam new", vim.inspect(config.title))
    assert(prompt.line() == "buy milk --path home/", prompt.line())
  end,

  ["anchored to a window the float spans its bottom edge, its title at the left"] = function()
    local anchor = vim.api.nvim_get_current_win()
    opened({ anchor = anchor })
    local config = vim.api.nvim_win_get_config(assert(prompt.window()))
    assert(config.relative == "win" and config.win == anchor and config.anchor == "SW", vim.inspect(config))
    assert(config.row == vim.api.nvim_win_get_height(anchor) - 1 and config.col == 0, vim.inspect(config))
    assert(config.width == vim.api.nvim_win_get_width(anchor) - 2, vim.inspect(config))
    assert(config.title_pos == "left", vim.inspect(config))
  end,

  ["with no window to anchor to the float is centred in the editor"] = function()
    opened()
    local config = vim.api.nvim_win_get_config(assert(prompt.window()))
    local left = config.col
    local right = vim.o.columns - (config.col + config.width + 2)
    assert(config.relative == "editor" and math.abs(left - right) <= 1, vim.inspect(config))
    assert(config.title_pos == "left", vim.inspect(config))
  end,

  ["after a pause the read runs once with the line and its answer is the footer"] = function()
    local seen = opened({}, "buy")
    prompt.more(" milk")
    prompt.footer_reads("read buy milk")
    assert(vim.deep_equal(seen.reads, { "buy milk" }), vim.inspect(seen.reads))
    local _, group = prompt.footer()
    assert(group == "FloatFooter", group)
  end,

  ["a newer read cancels the one still running"] = function()
    local seen = opened({}, "buy")
    wait_for(function()
      return #seen.reads == 1
    end)
    prompt.more(" milk")
    prompt.footer_reads("read buy milk")
    assert(seen.cancelled >= 1 and #seen.reads == 2, vim.inspect(seen))
  end,

  ["a refusal shows in DiagnosticError and <CR> sends nothing"] = function()
    local seen = opened({}, "buy --due never")
    prompt.footer_reads("cannot read never")
    local _, group = prompt.footer()
    assert(group == "DiagnosticError", group)
    prompt.more("<CR>")
    vim.wait(30)
    assert(#seen.submitted == 0 and prompt.window(), vim.inspect(seen))
  end,

  ["an answer too long for the footer is shown whole under the line, and the float grows to hold it"] = function()
    local long = string.rep("dam cannot read this ", 8)
    opened({
      read = function(text, done)
        done(text:find("long") and long or "short", text:find("long") ~= nil)
        return { cancel = function() end }
      end,
    }, "buy --due long")
    prompt.footer_reads(long)
    local win = assert(prompt.window())
    local buf = vim.api.nvim_win_get_buf(win)
    local below = {}
    for _, mark in ipairs(vim.api.nvim_buf_get_extmarks(buf, -1, 0, -1, { details = true })) do
      for _, line in ipairs(mark[4].virt_lines or {}) do
        table.insert(below, line[1][1])
        assert(line[1][2] == "DiagnosticError", vim.inspect(line))
      end
    end
    assert(vim.trim(table.concat(below, " ")) == vim.trim(long), vim.inspect(below))
    assert(vim.api.nvim_win_get_height(win) == 1 + #below, vim.api.nvim_win_get_height(win))
    prompt.more("<C-u>buy")
    prompt.footer_reads("short")
    assert(vim.api.nvim_win_get_height(win) == 1 and #vim.api.nvim_buf_get_extmarks(buf, -1, 0, -1, {}) == 0)
  end,

  ["<CR> judges a line not yet read, then submits it and closes"] = function()
    local seen = opened({}, "buy milk<CR>")
    wait_for(function()
      return #seen.submitted == 1
    end)
    assert(seen.submitted[1] == "buy milk" and prompt.window() == nil, vim.inspect(seen))
    assert(vim.api.nvim_get_mode().mode == "n", vim.api.nvim_get_mode().mode)
  end,

  ["<Esc> and <C-c> close without sending"] = function()
    for _, key in ipairs({ "<Esc>", "<C-c>" }) do
      local seen = opened({}, "buy milk" .. key)
      vim.wait(200)
      assert(prompt.window() == nil and #seen.submitted == 0, key)
      assert(vim.api.nvim_get_mode().mode == "n", vim.api.nvim_get_mode().mode)
    end
  end,

  ["<C-j> submits and stays open with the prefill again"] = function()
    local seen = opened({ prefill = "--path home/ " }, "buy milk<C-j>")
    wait_for(function()
      return #seen.submitted == 1
    end)
    assert(seen.submitted[1] == "--path home/ buy milk", vim.inspect(seen))
    assert(prompt.window() and prompt.line() == "--path home/ ", prompt.line())
  end,

  ["<Up> and <Down> walk this session's history of the same prompt"] = function()
    local seen, prompts = opened({}, "buy milk<CR>")
    wait_for(function()
      return #seen.submitted == 1
    end)
    prompt.open(function()
      prompts:open({ title = "dam new", prefill = "", read = function() end, submit = function() end })
    end, "pay<Up>")
    assert(prompt.line() == "buy milk", prompt.line())
    prompt.more("<Down>")
    assert(prompt.line() == "pay", prompt.line())
  end,

  ["completion takes the caller's candidates as you type"] = function()
    opened({
      complete = function(line, col)
        local start = line:sub(1, col):find("%S*$") - 1
        return start, { { word = "--due", menu = "Set the due date" }, { word = "--deadline", menu = "deadline" } }
      end,
    }, "buy --d")
    local buf = vim.api.nvim_win_get_buf(assert(prompt.window()))
    assert(vim.bo[buf].autocomplete and vim.bo[buf].complete == "F", vim.bo[buf].complete)
    local source = vim.b[buf].damnit_complete
    assert(source(1, "") == 4)
    local items = source(0, "--")
    assert(#items == 2 and items[1].word == "--due" and items[1].menu == "Set the due date", vim.inspect(items))
  end,
}
