# Test APL checker protection-flow validation.

List protection_sources = [
  "ASVInt hidden = 1 SASVInt vault = hidden VTime temporary = hidden ASVInt copy = temporary",
  "List items = [1:ASV, 2:SASV] SASVInt vault = 1",
  "ASVStr value = secret input",
  "ASVInt hidden = 1 AVInt public = hidden",
  "AVStr public = secret input",
  "SASVStr vault = input",
  "ASVInt hidden = 1 List items = [hidden:AV]",
  "ASVInt hidden = 1 out hidden",
  "ASVInt hidden = 1 VTime temporary = hidden AVInt public = temporary",
  "List items = [1] ASVInt hidden = 2 add(items, hidden) out items",
  "AVInt value = 1 secretup(value) secretup(value) secretup(value)"
]

out "Checker protection reports:"

pick(protection_sources): protection_source {
  VTime protection_report = bootstrap.compile_report(protection_source)
  out get(protection_report, 0)

  if get(protection_report, 0) == checker.STATUS_FAIL {
    out get(protection_report, 1)
  }
}

out "Done!"
