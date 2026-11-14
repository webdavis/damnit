return {
  schema = 1,
  remote = {
    todoist = { api_token = "..." },
    -- or: api_token = function() return os.getenv("TODOIST_API_TOKEN") end
  },
  token = {
    tool = {
      value = "damk_v1_...",
      expires = "2027-01-01T00:00:00Z",
      not_before = "2026-01-01T00:00:00Z",
      via = { "local" },
      require = { "session", "model" },
      grant = {
        { allow = { "task:read", "task:create" }, paths = { "work" }, labels = { "errand" }, own = true, remotes = { "todoist" } },
      },
    },
  },
}
