local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local filterbar = require("helpers.filterbar")
local prompt = require("helpers.prompt")
local ui = require("helpers.ui")

local EXIT_USAGE = 2

local function creates(fake)
  return vim.tbl_filter(function(call)
    return call.argv[3] == "create"
  end, fake.of("filter"))
end

return {
  ["<C-s> asks a name, saves the bar's flags with dam filter create, and the view becomes that filter"] = function()
    local _, fake = filterbar.opened()
    local cwd = fake.of("list")[1].cwd
    local notes = buffer.capture_notes()
    ui.answering({ input = "bugs" }, function(asked)
      filterbar.bar("<End> --overdue<C-s>")
      buffer.wait_for(function()
        return #notes > 0
      end)
      assert(asked[1] == "save as filter: ", vim.inspect(asked))
    end)
    local sent = creates(fake)
    local want =
      { "dam", "filter", "create", "bugs", "--due", "..today+7", "--labels", "bug", "--overdue", "-o", "json" }
    assert(#sent == 1 and vim.deep_equal(sent[1].argv, want) and sent[1].cwd == cwd, vim.inspect(sent))
    assert(notes[1].text == "saved filter bugs", vim.inspect(notes))
    assert(prompt.window() == nil, "the bar is still open")
    assert(vim.api.nvim_buf_get_name(0) == "dam://list/bugs", vim.api.nvim_buf_get_name(0))
  end,

  ["a refused save shows dam's message in the bar and nothing closes"] = function()
    local _, fake = filterbar.opened()
    fake.reply("filter", function(argv)
      if argv[3] == "create" then
        return fixtures.refusal("error-filter-create-taken.json", EXIT_USAGE)
      end
      return fixtures.answer("filter-list.json")
    end)
    filterbar.bar()
    prompt.footer_reads("3 rows")
    ui.answering({ input = "bugs" }, function()
      prompt.more("<C-s>")
    end)
    local message = fixtures.decoded("error-filter-create-taken.json").error.message
    prompt.footer_reads(message)
    assert(vim.api.nvim_buf_get_name(vim.fn.bufnr("#")) ~= "dam://list/bugs")
    assert(prompt.line() == "--due ..today+7 --labels bug", prompt.line())
  end,

  ["no name saves nothing and the bar stays"] = function()
    local _, fake = filterbar.opened()
    ui.answering({}, function()
      filterbar.bar("<C-s>")
    end)
    vim.wait(30)
    assert(#creates(fake) == 0 and prompt.window(), vim.inspect(creates(fake)))
  end,

  ["marks on the rows are kept through the bar"] = function()
    filterbar.opened()
    buffer.feed("m")
    assert(vim.wo.winbar:find("1 marked", 1, true), vim.wo.winbar)
    prompt.type("f<Esc>")
    buffer.wait_for(function()
      return prompt.window() == nil and vim.wo.winbar:find("1 marked", 1, true) ~= nil
    end)
  end,
}
