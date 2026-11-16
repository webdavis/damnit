return {
  owner = "Ada",
  done = { interactive = false },
  remote = {
    todoist = { url = "todoist::", deadline = "60s", path = "work" },
  },
  category = {
    effort = { values = { "light", "admin", "deep" }, exclusive = true },
  },
  filter = {
    today = { due = "..today" },
    review = { labels = { "bug", "not:waiting" }, any = { { priority = "1,2" }, { due = "..today" } } },
  },
}
