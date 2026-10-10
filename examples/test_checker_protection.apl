# Test APL checker protection-flow validation.

List protection_sources = [
  "ASVInt secret = 1 SASVInt vault = secret VTime temporary = secret ASVInt copy = temporary",
  "List items = [1:ASV, 2:SASV] SASVInt vault = 1",
  "ASVStr value = secret input",
  "ASVInt secret = 1 AVInt public = secret",
  "AVStr public = secret input",
  "SASVStr vault = input",
  "ASVInt secret = 1 List items = [secret:AV]",
  "ASVInt secret = 1 out secret",
  "ASVInt secret = 1 VTime temporary = secret AVInt public = temporary",
  "List items = [1] ASVInt secret = 2 add(items, secret) out items",
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
