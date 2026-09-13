AVInt x = 4
AVInt y = (x + 2) * 3
AVBool unchanged = x =self=
AVInt age = input
ASVStr token = "hidden"
ASVStr password = secret input
SASVStr master_key = "raw"
AVStr master_type = ""
AVStr master_protection = ""
List public_items = ["A", "P"]
List secret_items = [1:SASV, "abc":ASV]
List matrix = [[1, 2], [3, 4]]

func parse_age(raw) {
  VTime parsed = int(raw)

  if parsed == NONE {
    return NONE
  }

  return parsed
}

age = parse_age(str(age))
master_type, master_protection = info(master_key)
secretup(token)
add(public_items, "L")
add(secret_items, "hidden":ASV)

if (x < y) and (unchanged == true) {
  x += 1
} else if x =self= {
  x = 10
} else {
  x /= 2
}

if age != NONE {
  out age
} else {
  out "bad age"
}

while (x < 20) (100) {
  x += 1

  if x == 12 {
    continue
  }

  if x == 15 {
    break
  }
}

pick("APL"): ch {
  VTime current = ch
  current = 1
  out current
}

VTime first = get(public_items, 0)
VTime last = pop(public_items)
VTime cell = matrix[0][1]
VTime head = public_items[:2]
VTime every_second = public_items[::2]
out first
out last
out cell
out master_type
out master_protection
stop
