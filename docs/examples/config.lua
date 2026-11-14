return {
  owner = "Ada",
  done = { interactive = false },
  remote = {
    todoist = { url = "todoist::", stale = "15m", deadline = "60s", path = "work" },
  },
  category = {
    effort = { values = { "light", "admin", "deep" }, exclusive = true },
  },
  filter = {
    today = { query = "due:today | overdue" },
  },
}
