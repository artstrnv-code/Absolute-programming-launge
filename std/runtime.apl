# APL runtime prelude.
# These functions are deliberately written in APL so the language can begin
# moving toward self-hosted runtime behavior.

func apl.is_none(value) {
  return value == NONE
}

func apl.not_none(value) {
  return value != NONE
}

func apl.first(value) {
  return get(value, 0)
}

func apl.pow_int(base, exponent) {
  if exponent < 0 {
    return NONE
  }

  VTime result = 1
  VTime index = 0

  while (index < exponent) (-1) {
    result *= base
    index += 1
  }

  return result
}
