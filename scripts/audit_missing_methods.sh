#!/usr/bin/env bash
# Phase 13 Pass A: enumerate every C++ method definition under
# forgottenserver/src/ and check whether a snake_case Rust counterpart exists
# anywhere in forgottenserver-rust/crates/.
#
# Output (CSV on stdout): cpp_file,cpp_method,rust_candidate,found_in,confidence
#
# Confidence levels:
#   HIGH   — exact snake_case match found in a Rust fn definition
#   MED    — token match found anywhere in Rust source (probably called or aliased)
#   LOW    — nothing found
#
# Excludes destructors, copy/move ctors, operator overloads, anonymous lambdas.

set -u

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"
CPP_SRC="$ROOT/forgottenserver/src"
RUST_CRATES="$ROOT/crates"

snake_case() {
  # CamelCase / camelCase  →  snake_case (lowercase, underscore-separated)
  echo "$1" | sed -E 's/([a-z0-9])([A-Z])/\1_\2/g; s/([A-Z]+)([A-Z][a-z])/\1_\2/g' | tr '[:upper:]' '[:lower:]'
}

echo "cpp_file,cpp_method,rust_candidate,found_in,confidence"

# Find every .cpp file under src/ (excluding tests/ and otpch.cpp)
while IFS= read -r cpp; do
  rel="${cpp#$CPP_SRC/}"
  case "$rel" in
    tests/*|http/tests/*|otpch.cpp|main.cpp) continue ;;
  esac

  # Extract method definitions: lines like  "ReturnType ClassName::methodName("
  # We capture the method name (last token before the open paren after `::`).
  # Skip destructors (~Foo) and operator overloads.
  grep -hE '^[A-Za-z_][A-Za-z0-9_*&:<>, ]*[ \t][A-Za-z_][A-Za-z0-9_]*::[A-Za-z_][A-Za-z0-9_]*[ \t]*\(' "$cpp" 2>/dev/null \
    | sed -E 's/.*::([A-Za-z_][A-Za-z0-9_]*)[ \t]*\(.*/\1/' \
    | sort -u \
    | while IFS= read -r method; do
        [ -z "$method" ] && continue
        case "$method" in
          operator*|"if"|"while"|"for"|"switch"|"return"|"do") continue ;;
        esac
        snake=$(snake_case "$method")
        # HIGH: exact `fn snake(` declaration
        hi=$(grep -rln "fn ${snake}\\b" "$RUST_CRATES" 2>/dev/null | head -3 | tr '\n' ';' | sed 's/;$//')
        if [ -n "$hi" ]; then
          # If multiple matches, show first 3 paths
          printf '%s,%s,%s,%s,HIGH\n' "$rel" "$method" "$snake" "$(echo "$hi" | sed "s|$RUST_CRATES/||g")"
          continue
        fi
        # MED: any mention of the snake_case token
        med=$(grep -rln "\\b${snake}\\b" "$RUST_CRATES" 2>/dev/null | head -3 | tr '\n' ';' | sed 's/;$//')
        if [ -n "$med" ]; then
          printf '%s,%s,%s,%s,MED\n' "$rel" "$method" "$snake" "$(echo "$med" | sed "s|$RUST_CRATES/||g")"
          continue
        fi
        # LOW: not found anywhere
        printf '%s,%s,%s,,LOW\n' "$rel" "$method" "$snake"
      done

done < <(find "$CPP_SRC" -maxdepth 2 -type f -name '*.cpp' | sort)
