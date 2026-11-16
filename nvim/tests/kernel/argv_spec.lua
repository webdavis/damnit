local argv = require("damnit.kernel.argv")

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["words split on spaces and tabs"] = function()
    same(argv.words("buy  oat\tmilk --due fri"), { "buy", "oat", "milk", "--due", "fri" })
  end,

  ["double and single quotes keep spaces inside one word"] = function()
    same(argv.words([[--body "two words" --recurrence 'every 2 weeks']]), {
      "--body",
      "two words",
      "--recurrence",
      "every 2 weeks",
    })
  end,

  ["a backslash keeps the next character, outside quotes and inside double quotes"] = function()
    same(argv.words([[say\ hi "a \"quoted\" word" 'it\s']]), { "say hi", 'a "quoted" word', [[it\s]] })
  end,

  ["nothing is expanded"] = function()
    same(argv.words("~/notes $HOME *.lua `date`"), { "~/notes", "$HOME", "*.lua", "`date`" })
  end,

  ["an empty quoted word is a word"] = function()
    same(argv.words([[--body ""]]), { "--body", "" })
  end,

  ["an unclosed quote runs to the end of the line"] = function()
    same(argv.words([[buy "oat milk]]), { "buy", "oat milk" })
  end,

  ["tokens say where each word starts and whether the line ends inside a word"] = function()
    local tokens, open = argv.tokens([[buy "oat milk" --due]])
    same(tokens, { { text = "buy", from = 1 }, { text = "oat milk", from = 5 }, { text = "--due", from = 16 } })
    assert(open == true)
    local _, after_space = argv.tokens("buy ")
    assert(after_space == false)
    local none, empty_open = argv.tokens("")
    same(none, {})
    assert(empty_open == false)
  end,

  ["the subject is the words before the first word starting with --, joined"] = function()
    local subject, rest = argv.subject({ "buy", "oat", "milk", "--due", "fri", "now" })
    assert(subject == "buy oat milk", subject)
    same(rest, { "--due", "fri", "now" })
  end,

  ["a line that starts with a flag has no subject"] = function()
    local subject, rest = argv.subject({ "--parent", "abc" })
    assert(subject == "", subject)
    same(rest, { "--parent", "abc" })
  end,
}
