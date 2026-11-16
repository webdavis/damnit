local M = {}

---@class damnit.Token
---@field text string
---@field from integer

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
