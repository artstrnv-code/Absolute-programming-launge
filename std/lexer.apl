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
AVStr lexer.TOKEN_EOF = "Eof"

List lexer.KEYWORDS = ["include", "exclude", "if", "else", "while", "pick", "break", "continue",
    "func", "return", "true", "false", "and", "or", "not",
    "out", "input", "secret", "secretup", "info", "stop", "fail", "NONE",
    "VTime", "List", "AV", "ASV", "SASV",
    "AVInt", "AVFloat", "AVBool", "AVStr", "AVBytes", "AVJson",
    "ASVInt", "ASVFloat", "ASVBool", "ASVStr", "ASVBytes", "ASVJson",
    "SASVInt", "SASVFloat", "SASVBool", "SASVStr", "SASVBytes", "SASVJson"]

List lexer.DIGITS = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
List lexer.SINGLE_SYMBOLS = ["(", ")", "{", "}", "[", "]", ",", ":", "+", "-", "*", "/", "%", "=", "!", ">", "<"]
List lexer.DOUBLE_SYMBOLS = ["==", "!=", ">=", "<=", ">>", "<<", "+=", "-=", "*=", "/=", "%=", "**"]

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

func lexer.position_at(source, offset) {
  VTime line = 1
  VTime column = 1
  VTime index = 0

  while ((index < offset) and (index < len(source))) (-1) {
    if ord(source[index]) == 10 {
      line += 1
      column = 1
    } else {
      column += 1
    }

    index += 1
  }

  return [line, column]
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

func lexer.tokenize_core(source, include_offsets) {
  VTime tokens = []
  VTime offsets = []
  VTime chars = []
  VTime index = 0
  VTime chars_index = 0

  while (index < len(source)) (-1) {
    VTime ch = source[index]

    if ch == "#" {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        if include_offsets {
          add(offsets, chars_index)
        }
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
        if include_offsets {
          add(offsets, chars_index)
        }
        chars = []
      }

      index += 1
      continue
    }

    if lexer.is_quote(ch) {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        if include_offsets {
          add(offsets, chars_index)
        }
        chars = []
      }

      VTime string_index = index
      index += 1
      VTime text_chars = []

      while ((index < len(source)) and (lexer.is_quote(source[index]) == false)) (-1) {
        if source[index] == char(92) {
          VTime escape_index = index
          index += 1

          if index >= len(source) {
            add(tokens, lexer.make_token(lexer.TOKEN_ERROR, "unterminated escape sequence"))
            if include_offsets {
              add(offsets, escape_index)
            }
            return [tokens, offsets]
          }

          VTime escape = source[index]

          if escape == "n" {
            add(text_chars, char(10))
          } else if escape == "r" {
            add(text_chars, char(13))
          } else if escape == "t" {
            add(text_chars, char(9))
          } else if escape == "0" {
            add(text_chars, char(0))
          } else if escape == char(92) {
            add(text_chars, char(92))
          } else if lexer.is_quote(escape) {
            add(text_chars, char(34))
          } else {
            VTime message = join(["invalid escape sequence `", char(92), escape, "`"], "")
            add(tokens, lexer.make_token(lexer.TOKEN_ERROR, message))
            if include_offsets {
              add(offsets, escape_index)
            }
            return [tokens, offsets]
          }
        } else {
          add(text_chars, source[index])
        }

        index += 1
      }

      if index >= len(source) {
        add(tokens, lexer.make_token(lexer.TOKEN_ERROR, "unterminated string literal"))
        if include_offsets {
          add(offsets, string_index)
        }
        return [tokens, offsets]
      }

      VTime text = join(text_chars, "")
      add(tokens, lexer.make_token(lexer.TOKEN_STR, text))
      if include_offsets {
        add(offsets, string_index)
      }

      index += 1

      continue
    }

    if source[index:index + 6] == "=self=" {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        if include_offsets {
          add(offsets, chars_index)
        }
        chars = []
      }

      add(tokens, lexer.make_token(lexer.TOKEN_SYM, "=self="))
      if include_offsets {
        add(offsets, index)
      }
      index += 6
      continue
    }

    if lexer.is_double_symbol(source[index:index + 2]) {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        if include_offsets {
          add(offsets, chars_index)
        }
        chars = []
      }

      VTime symbol = source[index:index + 2]
      add(tokens, lexer.make_token(lexer.TOKEN_SYM, symbol))
      if include_offsets {
        add(offsets, index)
      }
      index += 2
      continue
    }

    if lexer.is_single_symbol(ch) {
      if len(chars) > 0 {
        VTime text = join(chars, "")
        add(tokens, lexer.make_token(lexer.token_type(text), text))
        if include_offsets {
          add(offsets, chars_index)
        }
        chars = []
      }

      add(tokens, lexer.make_token(lexer.TOKEN_SYM, ch))
      if include_offsets {
        add(offsets, index)
      }
      index += 1
      continue
    } else {
      if len(chars) == 0 {
        chars_index = index
      }

      add(chars, ch)
    }

    index += 1
  }

  if len(chars) > 0 {
    VTime text = join(chars, "")
    add(tokens, lexer.make_token(lexer.token_type(text), text))
    if include_offsets {
      add(offsets, chars_index)
    }
  }

  return [tokens, offsets]
}

func lexer.tokenize_with_offsets(source) {
  return lexer.tokenize_core(source, true)
}

func lexer.tokenize(source) {
  return get(lexer.tokenize_core(source, false), 0)
}
