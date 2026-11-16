local M = {}

---@class damnit.Token
---@field text string
---@field from integer
---@field quoted? boolean

---@param line string
---@return damnit.Token[] tokens, boolean open
function M.tokens(line)
  local tokens, word, quote, i = {}, nil, nil, 1
  while i <= #line do
    local char = line:sub(i, i)
    if quote == "'" and char ~= "'" then
      word.text = word.text .. char
    elseif char == "\\" and quote ~= "'" and i < #line then
      word = word or { text = "", from = i }
      i = i + 1
      word.text = word.text .. line:sub(i, i)
    elseif char == quote then
      quote = nil
    elseif not quote and (char == '"' or char == "'") then
      word = word or { text = "", from = i }
      word.quoted = true
      quote = char
    elseif not quote and char:match("%s") then
      table.insert(tokens, word)
      word = nil
    else
      word = word or { text = "", from = i }
      word.text = word.text .. char
    end
    i = i + 1
  end
  table.insert(tokens, word)
  return tokens, word ~= nil
end

---@param line string
---@return string[]
function M.words(line)
  return vim.tbl_map(function(token)
    return token.text
  end, (M.tokens(line)))
end

---@param line string
---@param flags damnit.Flag[] the command's flags
---@return string[] words, each value of several words joined into one
function M.args(line, flags)
  local named = {}
  for _, flag in ipairs(flags) do
    named[flag.name] = flag
  end
  local words, value = {}, nil
  for _, token in ipairs((M.tokens(line))) do
    local flag = not token.quoted and named[token.text]
    local stray = not token.quoted and not flag and vim.startswith(token.text, "--")
    if value and not flag and not stray then
      local first = words[value] == nil
      words[value] = first and token.text or (words[value] .. " " .. token.text)
      if first and token.quoted then
        value = nil
      end
    else
      table.insert(words, token.text)
      value = flag and flag.value and #words + 1 or nil
    end
  end
  return words
end

---@param words string[]
---@return string subject, string[] rest
function M.subject(words)
  for index, word in ipairs(words) do
    if vim.startswith(word, "--") then
      return table.concat(words, " ", 1, index - 1), vim.list_slice(words, index)
    end
  end
  return table.concat(words, " "), {}
end

return M
