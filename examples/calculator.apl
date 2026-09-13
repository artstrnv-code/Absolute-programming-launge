# Hard test: stack calculator sketch on the current APL base.
# Runtime is not implemented yet, so input tokenization is represented as a
# literal list. Later this should become something like:
# AVStr raw = input
# List mass = split(raw, " ")

List stacknum = []
List stackop = []
List mass = ["2", "+", "3", "*", "4"]

func priority(op) {
  if op == "**" {
    return 3
  } else if op == "/" {
    return 2
  } else if op == "*" {
    return 2
  } else if op == "+" {
    return 1
  } else if op == "-" {
    return 1
  } else {
    return NONE
  }
}

func apply(a, b, op) {
  if op == "+" {
    return a + b
  } else if op == "-" {
    return a - b
  } else if op == "*" {
    return a * b
  } else if op == "/" {
    return a / b
  } else if op == "**" {
    return NONE
  } else {
    return NONE
  }
}

func is_number(text) {
  VTime number = float(text)

  if number == NONE {
    return false
  } else {
    return true
  }
}

func collapse_once() {
  VTime op = pop(stackop)
  VTime b = float(pop(stacknum))
  VTime a = float(pop(stacknum))
  VTime result = apply(a, b, op)

  if result == NONE {
    return false
  }

  add(stacknum, str(result))
  return true
}

pick(mass): item {
  if is_number(item) == true {
    add(stacknum, item)
  } else if item == "(" {
    add(stackop, item)
  } else if item == ")" {
    VTime top = pop(stackop)

    while (top != NONE) (100) {
      if top == "(" {
        break
      }

      add(stackop, top)
      VTime ok = collapse_once()

      if ok == false {
        fail "bad expression"
      }

      top = pop(stackop)
    }
  } else if item == "+" {
    add(stackop, item)
  } else if item == "-" {
    add(stackop, item)
  } else if item == "*" {
    add(stackop, item)
  } else if item == "/" {
    add(stackop, item)
  } else if item == "**" {
    add(stackop, item)
  } else {
    fail "bad token"
  }
}

VTime pending = pop(stackop)

while (pending != NONE) (100) {
  add(stackop, pending)
  VTime ok = collapse_once()

  if ok == false {
    fail "bad expression"
  }

  pending = pop(stackop)
}

VTime answer = get(stacknum, 0)
VTime also_answer = stacknum[0]

if answer == NONE {
  fail "empty result"
} else {
  out answer
}

stop
