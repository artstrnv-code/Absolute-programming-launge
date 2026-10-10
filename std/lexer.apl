# APL lexer helpers.
# This file is part of the standard prelude and is compiled together with
# user code.

AVStr lexer.TOKEN_KW = "Keyword"
AVStr lexer.TOKEN_IDENT = "Ident"
AVStr lexer.TOKEN_INT = "Int"
AVStr lexer.TOKEN_FLOAT = "Float"
AVStr lexer.TOKEN_STR = "Str"
AVStr lexer.TOKEN_SYM = "Symbol"
AVStr lexer.TOKEN_ERROR = "Error"

List lexer.KEYWORDS = ["if", "else", "while", "pick", "break", "continue",
    "func", "return", "true", "false", "and", "or", "not",
    "out", "input", "secret", "secretup", "info", "stop", "fail", "NONE",
    "VTime", "List", "AV", "ASV", "SASV",
    "AVInt", "AVFloat", "AVBool", "AVStr", "AVBytes", "AVJson",
    "ASVInt", "ASVFloat", "ASVBool", "ASVStr", "ASVBytes", "ASVJson",
    "SASVInt", "SASVFloat", "SASVBool", "SASVStr", "SASVBytes", "SASVJson"]

List lexer.DIGITS = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
List lexer.SINGLE_SYMBOLS = ["(", ")", "{", "}", "[", "]", ",", ":", "+", "-", "*", "/", "=", "!", ">", "<"]
List lexer.DOUBLE_SYMBOLS = ["==", "!=", ">=", "<=", "+=", "-=", "*=", "/=", "**"]

func lexer.is_space(value) {
  if value == " " {
    return true
  }

  if ord(value) == 9 {
    return true
  }

  if ord(value) == 10 {
    return true
  }

  if ord(value) == 13 {
    return true
  }

  return false
}

func lexer.is_line_end(value) {
  if ord(value) == 10 {
    return true
  }

  if ord(value) == 13 {
    return true
  }

  return false
}

func lexer.is_quote(value) {
  return ord(value) == 34
}

func lexer.is_single_symbol(value) {
  pick(lexer.SINGLE_SYMBOLS): symbol {
    if symbol == value {
      return true
    }
  }

  return false
}

func lexer.is_double_symbol(value) {
  pick(lexer.DOUBLE_SYMBOLS): symbol {
    if symbol == value {
      return true
    }
  }

  return false
}

func lexer.make_token(kind, value) {
  return [kind, value]
}

func lexer.token_value(token) {
  return get(token, 1)
}

func lexer.token_kind(token) {
  return get(token, 0)
}

func lexer.is_keyword(value) {
  pick(lexer.KEYWORDS): keyword {
    if keyword == value {
      return true
    }
  }

  return false
}

func lexer.is_digit(value) {
  pick(lexer.DIGITS): digit {
    if digit == value {
      return true
    }
  }

  return false
}

func lexer.is_int(value) {
  if value == "" {
    return false
  }

  pick(value): ch {
    if lexer.is_digit(ch) == false {
      return false
    }
  }

  return true
}

func lexer.is_float(value) {
  VTime dot_count = 0
  VTime digit_count = 0
  VTime index = 0

  if value == "" {
    return false
  }

  pick(value): ch {
    if ch == "." {
      if index == 0 {
        return false
      }

      if index == len(value) - 1 {
        return false
      }

      dot_count += 1
    } else {
      if lexer.is_digit(ch) == false {
        return false
      }

      digit_count += 1
    }

    index += 1
  }

  if dot_count != 1 {
    return false
  }

  if digit_count == len(value) - 1 {
    return true
  }

  return false
}

func lexer.token_type(value) {
  if lexer.is_keyword(value) {
    return lexer.TOKEN_KW
  }

  if lexer.is_float(value) {
    return lexer.TOKEN_FLOAT
  }

  if lexer.is_int(value) {
    return lexer.TOKEN_INT
  }

  return lexer.TOKEN_IDENT
}

func lexer.tokenize(source) {
  VTime tokens = []
  VTime chars = []
  VTime index = 0

  while (index < len(source)) (-1) {
    VTime ch = source[index]

    if ch == "#" {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        chars = []
      }

      while ((index < len(source)) and (lexer.is_line_end(source[index]) == false)) (-1) {
        index += 1
      }

      continue
    }

    if lexer.is_space(ch) {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        chars = []
      }

      index += 1
      continue
    }

    if lexer.is_quote(ch) {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        chars = []
      }

      index += 1
      VTime text_chars = []

      while ((index < len(source)) and (lexer.is_quote(source[index]) == false)) (-1) {
        add(text_chars, source[index])
        index += 1
      }

      if index >= len(source) {
        add(tokens, lexer.make_token(lexer.TOKEN_ERROR, "unterminated string literal"))
        return tokens
      }

      VTime text = join(text_chars, "")
      add(tokens, lexer.make_token(lexer.TOKEN_STR, text))

      index += 1

      continue
    }

    if source[index:index + 6] == "=self=" {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        chars = []
      }

      add(tokens, lexer.make_token(lexer.TOKEN_SYM, "=self="))
      index += 6
      continue
    }

    if lexer.is_double_symbol(source[index:index + 2]) {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        chars = []
      }

      VTime symbol = source[index:index + 2]
      add(tokens, lexer.make_token(lexer.TOKEN_SYM, symbol))
      index += 2
      continue
    }

    if lexer.is_single_symbol(ch) {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        chars = []
      }

      add(tokens, lexer.make_token(lexer.TOKEN_SYM, ch))
      index += 1
      continue
    } else {
      add(chars, ch)
    }

    index += 1
  }

  if len(chars) > 0 {
    VTime text = join(chars, "")
    add(tokens, lexer.make_token(lexer.token_type(text), text))
  }

  return tokens
}
