#!/bin/bash
# ============================================================================
# omnicalc Destructive Test Suite
# Tests 200+ edge cases across every subsystem
# Output: structured log with PASS/FAIL/CRASH for every case
# ============================================================================

BINARY="/home/sagnik/code/omnicalc/target/release/omnicalc"
LOG="/home/sagnik/code/omnicalc/test_results.log"
PASS_COUNT=0
FAIL_COUNT=0
CRASH_COUNT=0
TOTAL=0

# Reset log
echo "============================================================================" > "$LOG"
echo "OMNICALC DESTRUCTIVE TEST LOG" >> "$LOG"
echo "Date: $(date -Iseconds)" >> "$LOG"
echo "Binary: $BINARY" >> "$LOG"
echo "Binary size: $(stat -c%s "$BINARY") bytes" >> "$LOG"
echo "============================================================================" >> "$LOG"
echo "" >> "$LOG"

run_test() {
    local SECTION="$1"
    local DESC="$2"
    local INPUT="$3"
    local EXPECT_STATUS="$4"  # 0 = expect success, 1 = expect error, any = either
    local EXPECT_CONTAINS="$5"  # substring expected in output (optional)
    local RAW_FLAG="$6"  # additional flags

    TOTAL=$((TOTAL + 1))
    local TEST_ID=$(printf "T%03d" "$TOTAL")

    # Run with timeout to catch hangs
    local OUTPUT
    local EXIT_CODE
    if [ -n "$RAW_FLAG" ]; then
        OUTPUT=$(timeout 5 "$BINARY" $RAW_FLAG "$INPUT" 2>&1)
        EXIT_CODE=$?
    else
        OUTPUT=$(timeout 5 "$BINARY" "$INPUT" 2>&1)
        EXIT_CODE=$?
    fi

    # Timeout => crash
    if [ "$EXIT_CODE" -eq 124 ]; then
        CRASH_COUNT=$((CRASH_COUNT + 1))
        echo "[$TEST_ID] HANG    [$SECTION] $DESC" >> "$LOG"
        echo "         Input:    $INPUT" >> "$LOG"
        echo "         Result:   TIMEOUT (5s)" >> "$LOG"
        echo "" >> "$LOG"
        return
    fi

    # Segfault / signal
    if [ "$EXIT_CODE" -gt 128 ]; then
        CRASH_COUNT=$((CRASH_COUNT + 1))
        local SIG=$((EXIT_CODE - 128))
        echo "[$TEST_ID] CRASH   [$SECTION] $DESC" >> "$LOG"
        echo "         Input:    $INPUT" >> "$LOG"
        echo "         Signal:   $SIG" >> "$LOG"
        echo "         Output:   $OUTPUT" >> "$LOG"
        echo "" >> "$LOG"
        return
    fi

    # Check status
    local STATUS_OK=true
    if [ "$EXPECT_STATUS" = "0" ] && [ "$EXIT_CODE" -ne 0 ]; then
        STATUS_OK=false
    fi
    if [ "$EXPECT_STATUS" = "1" ] && [ "$EXIT_CODE" -eq 0 ]; then
        STATUS_OK=false
    fi

    # Check content
    local CONTENT_OK=true
    if [ -n "$EXPECT_CONTAINS" ]; then
        if ! echo "$OUTPUT" | grep -qF -- "$EXPECT_CONTAINS"; then
            CONTENT_OK=false
        fi
    fi

    if $STATUS_OK && $CONTENT_OK; then
        PASS_COUNT=$((PASS_COUNT + 1))
        echo "[$TEST_ID] PASS    [$SECTION] $DESC" >> "$LOG"
        echo "         Input:    $INPUT" >> "$LOG"
        echo "         Output:   $OUTPUT" >> "$LOG"
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        echo "[$TEST_ID] FAIL    [$SECTION] $DESC" >> "$LOG"
        echo "         Input:    $INPUT" >> "$LOG"
        echo "         Exit:     $EXIT_CODE (expected $EXPECT_STATUS)" >> "$LOG"
        echo "         Output:   $OUTPUT" >> "$LOG"
        if [ -n "$EXPECT_CONTAINS" ]; then
            echo "         Expected: '$EXPECT_CONTAINS'" >> "$LOG"
        fi
    fi
    echo "" >> "$LOG"
}

run_pipe_test() {
    local SECTION="$1"
    local DESC="$2"
    local PIPE_INPUT="$3"
    local EXPECT_STATUS="$4"
    local EXPECT_CONTAINS="$5"
    local EXTRA_FLAGS="$6"

    TOTAL=$((TOTAL + 1))
    local TEST_ID=$(printf "T%03d" "$TOTAL")

    local OUTPUT
    local EXIT_CODE
    OUTPUT=$(echo "$PIPE_INPUT" | timeout 5 "$BINARY" $EXTRA_FLAGS 2>&1)
    EXIT_CODE=$?

    if [ "$EXIT_CODE" -eq 124 ]; then
        CRASH_COUNT=$((CRASH_COUNT + 1))
        echo "[$TEST_ID] HANG    [$SECTION] $DESC" >> "$LOG"
        echo "         Pipe:     $PIPE_INPUT" >> "$LOG"
        echo "" >> "$LOG"
        return
    fi

    if [ "$EXIT_CODE" -gt 128 ]; then
        CRASH_COUNT=$((CRASH_COUNT + 1))
        local SIG=$((EXIT_CODE - 128))
        echo "[$TEST_ID] CRASH   [$SECTION] $DESC" >> "$LOG"
        echo "         Pipe:     $PIPE_INPUT" >> "$LOG"
        echo "         Signal:   $SIG" >> "$LOG"
        echo "" >> "$LOG"
        return
    fi

    local STATUS_OK=true
    if [ "$EXPECT_STATUS" = "0" ] && [ "$EXIT_CODE" -ne 0 ]; then
        STATUS_OK=false
    fi
    if [ "$EXPECT_STATUS" = "1" ] && [ "$EXIT_CODE" -eq 0 ]; then
        STATUS_OK=false
    fi

    local CONTENT_OK=true
    if [ -n "$EXPECT_CONTAINS" ]; then
        if ! echo "$OUTPUT" | grep -qF -- "$EXPECT_CONTAINS"; then
            CONTENT_OK=false
        fi
    fi

    if $STATUS_OK && $CONTENT_OK; then
        PASS_COUNT=$((PASS_COUNT + 1))
        echo "[$TEST_ID] PASS    [$SECTION] $DESC" >> "$LOG"
        echo "         Pipe:     $PIPE_INPUT" >> "$LOG"
        echo "         Output:   $OUTPUT" >> "$LOG"
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        echo "[$TEST_ID] FAIL    [$SECTION] $DESC" >> "$LOG"
        echo "         Pipe:     $PIPE_INPUT" >> "$LOG"
        echo "         Exit:     $EXIT_CODE (expected $EXPECT_STATUS)" >> "$LOG"
        echo "         Output:   $OUTPUT" >> "$LOG"
        if [ -n "$EXPECT_CONTAINS" ]; then
            echo "         Expected: '$EXPECT_CONTAINS'" >> "$LOG"
        fi
    fi
    echo "" >> "$LOG"
}


# ==========================================================================
# SECTION 1: EMPTY / DEGENERATE INPUT
# ==========================================================================
echo "--- Section 1: Empty & Degenerate Input ---" >> "$LOG"
echo "" >> "$LOG"

run_test "EMPTY" "Completely empty string" "" "1" ""
run_test "EMPTY" "Whitespace only" "   " "1" ""
run_test "EMPTY" "Single newline" $'\n' "any" ""
run_test "EMPTY" "Tab characters only" $'\t\t' "any" ""

# ==========================================================================
# SECTION 2: BASIC ARITHMETIC (EXACT RATIONAL)
# ==========================================================================
echo "--- Section 2: Basic Arithmetic ---" >> "$LOG"
echo "" >> "$LOG"

run_test "ARITH" "Simple addition" "2 + 3" "0" "5"
run_test "ARITH" "Simple subtraction" "10 - 7" "0" "3"
run_test "ARITH" "Simple multiplication" "6 * 7" "0" "42"
run_test "ARITH" "Simple division exact" "10 / 2" "0" "5"
run_test "ARITH" "Rational result" "1 / 3" "0" "1/3"
run_test "ARITH" "Negative number" "-5" "0" "-5"
run_test "ARITH" "Double negative" "--5" "0" "5"
run_test "ARITH" "Subtraction yielding negative" "3 - 10" "0" "-7"
run_test "ARITH" "Zero" "0" "0" "0"
run_test "ARITH" "Modulus" "10 % 3" "0" "1"
run_test "ARITH" "Modulus zero divisor" "10 % 0" "1" "Division by zero"
run_test "ARITH" "Power" "2^10" "0" "1024"
run_test "ARITH" "Power zero" "5^0" "0" "1"
run_test "ARITH" "Power of zero" "0^5" "0" "0"
run_test "ARITH" "Zero to zero (0^0)" "0^0" "0" "1"
run_test "ARITH" "Negative power" "2^-3" "0" "1/8"
run_test "ARITH" "Chained operations" "2 + 3 * 4" "0" "14"
run_test "ARITH" "Left paren grouping" "(2 + 3) * 4" "0" "20"
run_test "ARITH" "Nested parens" "((2 + 3) * (4 + 5))" "0" "45"
run_test "ARITH" "Deeply nested parens" "((((((1))))))" "0" "1"
run_test "ARITH" "Unbalanced left paren" "(2 + 3" "1" ""
run_test "ARITH" "Unbalanced right paren" "2 + 3)" "1" ""
run_test "ARITH" "Division by zero" "1 / 0" "1" ""
run_test "ARITH" "Large integer" "999999999999999999 + 1" "0" "1000000000000000000"
run_test "ARITH" "Very large power" "2^64" "0" "18446744073709551616"
run_test "ARITH" "Huge power (potential hang)" "2^1000" "0" ""
run_test "ARITH" "Negative * negative" "-3 * -4" "0" "12"
run_test "ARITH" "Double star power" "2**10" "0" "1024"

# ==========================================================================
# SECTION 3: FLOATING POINT EDGE CASES
# ==========================================================================
echo "--- Section 3: Floating Point ---" >> "$LOG"
echo "" >> "$LOG"

run_test "FLOAT" "Basic float" "1.5 + 2.5" "0" "4"
run_test "FLOAT" "Very small float" "1e-300 + 1e-300" "0" ""
run_test "FLOAT" "Very large float" "1e308" "0" ""
run_test "FLOAT" "Overflow float" "1e308 * 10" "0" "" # inf?
run_test "FLOAT" "Underflow float" "1e-308 / 1e308" "0" ""
run_test "FLOAT" "Scientific notation" "6.022e23" "0" ""
run_test "FLOAT" "Negative exponent" "1.6e-19" "0" ""
run_test "FLOAT" "Float precision 0.1+0.2" "0.1 + 0.2" "0" ""
run_test "FLOAT" "Leading decimal point" ".5 + .5" "any" ""
run_test "FLOAT" "Number with underscores" "1_000_000" "0" "1000000"
run_test "FLOAT" "Only underscore in number" "1__2" "0" "12"

# ==========================================================================
# SECTION 4: BASE CONVERSIONS
# ==========================================================================
echo "--- Section 4: Base Conversions ---" >> "$LOG"
echo "" >> "$LOG"

run_test "BASE" "Hex input" "0xFF" "0" "255"
run_test "BASE" "Binary input" "0b1010" "0" "10"
run_test "BASE" "Octal input" "0o77" "0" "63"
run_test "BASE" "To hex" "255 to hex" "0" "0xff"
run_test "BASE" "To bin" "10 to bin" "0" "0b1010"
run_test "BASE" "To oct" "63 to oct" "0" "0o77"
run_test "BASE" "To dec (identity)" "42 to dec" "0" "42"
run_test "BASE" "Hex arithmetic" "0xFF + 1" "0" "256"
run_test "BASE" "Binary arithmetic" "0b1010 + 0b0101" "0" "15"
run_test "BASE" "Empty hex 0x" "0x" "any" ""
run_test "BASE" "Empty binary 0b" "0b" "any" ""
run_test "BASE" "Empty octal 0o" "0o" "any" ""
run_test "BASE" "Hex with underscores" "0xFF_FF" "0" "65535"
run_test "BASE" "Negative to hex" "-1 to hex" "0" ""
run_test "BASE" "Global base flag hex" "255" "0" "0xff" "-b hex"
run_test "BASE" "Global base flag bin" "10" "0" "0b1010" "-b bin"

# ==========================================================================
# SECTION 5: BITWISE OPERATIONS
# ==========================================================================
echo "--- Section 5: Bitwise ---" >> "$LOG"
echo "" >> "$LOG"

run_test "BITWISE" "AND" "0xFF & 0x0F" "0" "15"
run_test "BITWISE" "OR" "0xF0 | 0x0F" "0" "255"
run_test "BITWISE" "XOR via caret" "0xFF ^ 0x0F" "0" "" # conflicts with power?
run_test "BITWISE" "Left shift" "1 << 10" "0" "1024"
run_test "BITWISE" "Right shift" "1024 >> 5" "0" "32"
run_test "BITWISE" "NOT (tilde)" "~0" "0" "-1"
run_test "BITWISE" "NOT 255" "~255" "0" "-256"
run_test "BITWISE" "Shift by 0" "42 << 0" "0" "42"
run_test "BITWISE" "Shift by 63" "1 << 63" "0" ""
run_test "BITWISE" "Large shift (overflow)" "1 << 64" "any" ""  # May panic

# ==========================================================================
# SECTION 6: UNIT CONVERSIONS
# ==========================================================================
echo "--- Section 6: Unit Conversions ---" >> "$LOG"
echo "" >> "$LOG"

run_test "UNITS" "Meters to feet" "1 m to ft" "0" ""
run_test "UNITS" "Feet to meters" "1 ft to m" "0" ""
run_test "UNITS" "Km to miles" "1 km to mi" "0" ""
run_test "UNITS" "Celsius to Fahrenheit" "100 degC to degF" "0" "212"
run_test "UNITS" "Fahrenheit to Celsius" "32 degF to degC" "0" "0"
run_test "UNITS" "Celsius zero" "0 degC to K" "0" "273.15"
run_test "UNITS" "Absolute zero" "-273.15 degC to K" "0" "0"
run_test "UNITS" "Kg to pounds" "1 kg to lb" "0" ""
run_test "UNITS" "Joules to calories" "1 J to cal" "0" ""
run_test "UNITS" "KWh to joules" "1 kWh to J" "0" "3600000"
run_test "UNITS" "Watts to horsepower" "746 W to hp" "0" ""
run_test "UNITS" "Atm to pascal" "1 atm to Pa" "0" "101325"
run_test "UNITS" "Data: MB to GiB" "1024 MB to GiB" "0" ""
run_test "UNITS" "Data: megabit" "1 megabit to bit" "0" "1000000"
run_test "UNITS" "Lightspeed constant c" "1 ly to km" "0" ""
run_test "UNITS" "Speed: kph to mph" "100 kph to mph" "0" ""
run_test "UNITS" "Feet-inches to cm" "5'10\" to cm" "0" "177.8"
run_test "UNITS" "Currency EUR to USD" "1 EUR to USD" "0" "1.038"
run_test "UNITS" "Currency INR to USD" "85.5 INR to USD" "0" ""
run_test "UNITS" "Unknown unit" "1 flurb to m" "1" "Unknown"
run_test "UNITS" "Dimensional mismatch" "1 m to s" "1" "mismatch"
run_test "UNITS" "Dimensional mismatch add" "5 m + 3 kg" "1" "mismatch"
run_test "UNITS" "Zero conversion" "0 km to m" "0" "0"
run_test "UNITS" "Negative temperature" "-40 degC to degF" "0" "-40"
run_test "UNITS" "Data rate bandwidth" "1 Mbps * 60 s to megabit" "0" "60"
run_test "UNITS" "Compound: speed * time" "60 kph * 2 hr to km" "0" ""

# ==========================================================================
# SECTION 7: UNCERTAINTY PROPAGATION
# ==========================================================================
echo "--- Section 7: Uncertainty ---" >> "$LOG"
echo "" >> "$LOG"

run_test "UNCERT" "Simple uncertainty" "5±0.1" "0" "5.0000±0.1000"
run_test "UNCERT" "Uncertainty addition" "(5±0.1) + (3±0.2)" "0" ""
run_test "UNCERT" "Uncertainty multiplication" "(5±0.1) * (3±0.2)" "0" ""
run_test "UNCERT" "Uncertainty division" "(10±0.5) / (2±0.1)" "0" ""
run_test "UNCERT" "Zero uncertainty" "5±0" "0" ""
run_test "UNCERT" "Negative nominal" "-5±0.1" "0" ""
run_test "UNCERT" "Uncertainty with units" "(2±0.02 J) / (523±5 W) to ms" "0" ""
run_test "UNCERT" "Uncertainty +/- syntax" "10+/-0.5" "0" ""
run_test "UNCERT" "Zero nominal with error" "0±0.5" "0" ""
run_test "UNCERT" "Very large error" "1±1e10" "0" ""

# ==========================================================================
# SECTION 8: SYMBOLIC DIFFERENTIATION
# ==========================================================================
echo "--- Section 8: Symbolic Differentiation ---" >> "$LOG"
echo "" >> "$LOG"

run_test "DIFF" "Linear" "diff(3x)" "0" "3"
run_test "DIFF" "Quadratic" "diff(x^2)" "0" "2x"
run_test "DIFF" "Cubic" "diff(x^3)" "0" "3 * x ^ 2"
run_test "DIFF" "Polynomial" "diff(6x^2)" "0" "12x"
run_test "DIFF" "Constant" "diff(42)" "0" "0"
run_test "DIFF" "sin" "diff(sin(x))" "0" "cos"
run_test "DIFF" "cos" "diff(cos(x))" "0" "sin"
run_test "DIFF" "exp" "diff(exp(x))" "0" "exp"
run_test "DIFF" "ln" "diff(ln(x))" "0" ""
run_test "DIFF" "Sum" "diff(x^2 + 3x + 1)" "0" ""
run_test "DIFF" "Product" "diff(x * sin(x))" "0" ""
run_test "DIFF" "Quotient" "diff(x / sin(x))" "0" ""
run_test "DIFF" "Chain rule" "diff(sin(x^2))" "0" ""
run_test "DIFF" "No variable (all constant)" "diff(5 + 3)" "0" "0"

# ==========================================================================
# SECTION 9: NUMERICAL INTEGRATION
# ==========================================================================
echo "--- Section 9: Integration ---" >> "$LOG"
echo "" >> "$LOG"

run_test "INTEG" "Polynomial" "integrate(x^2; 0; 1)" "0" ""
run_test "INTEG" "Known integral 6x^2" "integrate(6x^2; 1; 5)" "0" "248"
run_test "INTEG" "Sin from 0 to pi" "integrate(sin(x); 0; 3.14159265)" "0" ""
run_test "INTEG" "Constant function" "integrate(1; 0; 10)" "0" "10"
run_test "INTEG" "Negative bounds" "integrate(x; -5; 5)" "0" "0"
run_test "INTEG" "Reversed bounds" "integrate(x^2; 5; 1)" "0" ""
run_test "INTEG" "Zero-width interval" "integrate(x; 3; 3)" "0" "0"
run_test "INTEG" "1 arg only (error)" "integrate(x)" "1" ""
run_test "INTEG" "2 args only (error)" "integrate(x; 0)" "1" ""
run_test "INTEG" "With explicit var" "integrate(x^2; x; 0; 1)" "0" ""

# ==========================================================================
# SECTION 10: EQUATION SOLVING
# ==========================================================================
echo "--- Section 10: Equation Solving ---" >> "$LOG"
echo "" >> "$LOG"

run_test "SOLVE" "Linear 2x=10" "2x = 10" "0" "x = 5"
run_test "SOLVE" "Linear 2x+4=10" "2x + 4 = 10" "0" "x = 3"
run_test "SOLVE" "Linear x-7=0" "x - 7 = 0" "0" "x = 7"
run_test "SOLVE" "Zero LHS" "0 = x" "0" ""
run_test "SOLVE" "Identity 0=0" "0 = 0" "any" ""
run_test "SOLVE" "Contradiction 1=2" "1 = 2" "any" ""
run_test "SOLVE" "Quadratic x^2=4" "x^2 = 4" "0" ""

# ==========================================================================
# SECTION 11: FACTORING
# ==========================================================================
echo "--- Section 11: Factoring ---" >> "$LOG"
echo "" >> "$LOG"

run_test "FACTOR" "Difference of squares" "factor(x^2 - 9)" "0" "(x - 3)(x + 3)"
run_test "FACTOR" "Perfect square" "factor(x^2 - 6x + 9)" "0" ""
run_test "FACTOR" "No real roots" "factor(x^2 + 1)" "0" ""
run_test "FACTOR" "Linear" "factor(2x + 4)" "0" ""
run_test "FACTOR" "Constant" "factor(42)" "0" ""

# ==========================================================================
# SECTION 12: MATH FUNCTIONS
# ==========================================================================
echo "--- Section 12: Math Functions ---" >> "$LOG"
echo "" >> "$LOG"

run_test "FUNC" "sqrt(4)" "sqrt(4)" "0" "2"
run_test "FUNC" "sqrt(2)" "sqrt(2)" "0" "1.414"
run_test "FUNC" "sqrt(-1) => complex" "sqrt(-1)" "0" "i"
run_test "FUNC" "sqrt(0)" "sqrt(0)" "0" "0"
run_test "FUNC" "cbrt(27)" "cbrt(27)" "0" "3"
run_test "FUNC" "cbrt(-8)" "cbrt(-8)" "0" "-2"
run_test "FUNC" "sin(0)" "sin(0)" "0" "0"
run_test "FUNC" "cos(0)" "cos(0)" "0" "1"
run_test "FUNC" "tan(0)" "tan(0)" "0" "0"
run_test "FUNC" "sin(pi)" "sin(pi)" "0" ""
run_test "FUNC" "cos(pi)" "cos(pi)" "0" "-1"
run_test "FUNC" "asin(1)" "asin(1)" "0" ""
run_test "FUNC" "acos(0)" "acos(0)" "0" ""
run_test "FUNC" "asin(2) domain error" "asin(2)" "1" "Domain error"
run_test "FUNC" "exp(0)" "exp(0)" "0" "1"
run_test "FUNC" "exp(1)" "exp(1)" "0" "2.718"
run_test "FUNC" "ln(1)" "ln(1)" "0" "0"
run_test "FUNC" "ln(e)" "ln(e)" "0" "1"
run_test "FUNC" "ln(0)" "ln(0)" "0" "" # -inf or complex
run_test "FUNC" "ln(-1)" "ln(-1)" "0" "" # complex
run_test "FUNC" "log10(100)" "log10(100)" "0" "2"
run_test "FUNC" "log10(0)" "log10(0)" "0" "" # -inf
run_test "FUNC" "abs(-5)" "abs(-5)" "0" "5"
run_test "FUNC" "abs(5)" "abs(5)" "0" "5"
run_test "FUNC" "round(2.7)" "round(2.7)" "0" "3"
run_test "FUNC" "floor(2.9)" "floor(2.9)" "0" "2"
run_test "FUNC" "ceil(2.1)" "ceil(2.1)" "0" "3"
run_test "FUNC" "normdist(0)" "normdist(0)" "0" "0.5"
run_test "FUNC" "No-arg function" "sin()" "1" "requires"
run_test "FUNC" "Unknown function" "bogus(5)" "1" "Unknown"

# ==========================================================================
# SECTION 13: COMPLEX NUMBERS
# ==========================================================================
echo "--- Section 13: Complex Numbers ---" >> "$LOG"
echo "" >> "$LOG"

run_test "COMPLEX" "i literal" "i" "0" ""
run_test "COMPLEX" "i squared" "i * i" "0" "" # should be -1
run_test "COMPLEX" "i^2" "i^2" "0" "" # -1
run_test "COMPLEX" "Complex addition" "(3 + 2i) + (1 + 4i)" "any" ""  # may not parse
run_test "COMPLEX" "sqrt(-4)" "sqrt(-4)" "0" "2"

# ==========================================================================
# SECTION 14: CONSTANTS
# ==========================================================================
echo "--- Section 14: Constants ---" >> "$LOG"
echo "" >> "$LOG"

run_test "CONST" "pi value" "pi" "0" "3.14159"
run_test "CONST" "tau value" "tau" "0" "6.28318"
run_test "CONST" "e value" "e" "0" "2.71828"
run_test "CONST" "pi Unicode π" "π" "0" "3.14159"
run_test "CONST" "tau Unicode τ" "τ" "0" "6.28318"
run_test "CONST" "pi/2" "pi / 2" "0" "1.5707"

# ==========================================================================
# SECTION 15: PERCENT & OF
# ==========================================================================
echo "--- Section 15: Percent & Of ---" >> "$LOG"
echo "" >> "$LOG"

run_test "PERCENT" "50 percent" "50%" "0" "1/2"
run_test "PERCENT" "5% of 200" "5% of 200" "0" "10"
run_test "PERCENT" "100% of 50" "100% of 50" "0" "50"
run_test "PERCENT" "0% of 100" "0% of 100" "0" "0"
run_test "PERCENT" "150% of 100" "150% of 100" "0" "150"
run_test "PERCENT" "Decimal percent 12.5% of 80" "12.5% of 80" "0" "10"

# ==========================================================================
# SECTION 16: IMPLICIT MULTIPLICATION
# ==========================================================================
echo "--- Section 16: Implicit Multiplication ---" >> "$LOG"
echo "" >> "$LOG"

run_test "IMPL_MUL" "2pi" "2pi" "0" "6.28318"
run_test "IMPL_MUL" "3(4+5)" "3(4 + 5)" "0" "27"
run_test "IMPL_MUL" "Number unit: 5 m" "5 m to cm" "0" "500"
run_test "IMPL_MUL" "Adjacent parens (2)(3)" "(2)(3)" "0" "6"
run_test "IMPL_MUL" "2sin(0)" "2sin(0)" "0" "0"
run_test "IMPL_MUL" "Ambiguous: 2x where x unknown" "2x" "0" "2x"

# ==========================================================================
# SECTION 17: PARSER EDGE CASES / MALFORMED INPUT
# ==========================================================================
echo "--- Section 17: Parser Edge Cases ---" >> "$LOG"
echo "" >> "$LOG"

run_test "PARSE" "Only operator +" "+" "1" ""
run_test "PARSE" "Only operator *" "*" "1" ""
run_test "PARSE" "Only operator /" "/" "1" ""
run_test "PARSE" "Double operator ++" "5 ++ 3" "any" ""
run_test "PARSE" "Trailing operator" "5 +" "1" ""
run_test "PARSE" "Leading operator *" "* 5" "1" ""
run_test "PARSE" "Multiple dots" "1.2.3" "any" ""
run_test "PARSE" "Only dot" "." "1" ""
run_test "PARSE" "Exclamation mark" "5!" "1" ""
run_test "PARSE" "At sign" "@" "1" ""
run_test "PARSE" "Hash" "#" "1" ""
run_test "PARSE" "Dollar sign" "$5" "1" ""
run_test "PARSE" "Backslash" "\\" "1" ""
run_test "PARSE" "Curly braces" "{1+2}" "1" ""
run_test "PARSE" "Square brackets" "[1+2]" "1" ""
run_test "PARSE" "Very long expression" "1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1" "0" "50"
run_test "PARSE" "Unicode emoji" "😀" "1" ""
run_test "PARSE" "Mixed valid/invalid" "2 + @ + 3" "1" ""
run_test "PARSE" "Semicolon at top level" "5; 3" "1" ""
run_test "PARSE" "Multiple equals" "x = y = 5" "any" ""

# ==========================================================================
# SECTION 18: CLI FLAGS & PIPE BEHAVIOR
# ==========================================================================
echo "--- Section 18: CLI Flags & Piping ---" >> "$LOG"
echo "" >> "$LOG"

run_test "CLI" "Eval flag -e" "2 + 3" "0" "5" "-e"
run_test "CLI" "Raw flag" "1/3" "0" "0.333333" "--raw"
run_test "CLI" "Raw with units" "5 m / (3 s)" "0" "1.6666" "--raw"
run_test "CLI" "Version" "" "0" "2026.1.0" "--version"
run_test "CLI" "Help" "" "0" "omnicalc" "--help"

run_pipe_test "PIPE" "Stdin single line" "2 + 3" "0" "5"
run_pipe_test "PIPE" "Stdin multi-line" "2 + 3
4 * 5" "0" "5"
run_pipe_test "PIPE" "Stdin with comments" "# comment
2 + 3" "0" "5"
run_pipe_test "PIPE" "Stdin empty lines" "
2 + 3
" "0" "5"
run_pipe_test "PIPE" "Stdin error mid-stream" "2 + 3
1 / 0
4 + 5" "1" ""
run_pipe_test "PIPE" "Stdin with raw flag" "1/3" "0" "0.333333" "--raw"
run_pipe_test "PIPE" "Stdin empty" "" "0" ""

# ==========================================================================
# SECTION 19: TEMPERATURE CONVERSION CORRECTNESS
# ==========================================================================
echo "--- Section 19: Temperature Conversion Deep ---" >> "$LOG"
echo "" >> "$LOG"

run_test "TEMP" "Boiling point C->F" "100 degC to degF" "0" "212"
run_test "TEMP" "Freezing point C->F" "0 degC to degF" "0" "32"
run_test "TEMP" "Body temp F->C" "98.6 degF to degC" "0" "37"
run_test "TEMP" "Kelvin to Celsius" "373.15 K to degC" "0" "100"
run_test "TEMP" "Celsius to Kelvin" "0 degC to K" "0" "273.15"
run_test "TEMP" "-40 crossover" "-40 degC to degF" "0" "-40"
run_test "TEMP" "-40 F to C crossover" "-40 degF to degC" "0" "-40"

# ==========================================================================
# SECTION 20: DIMENSIONAL ANALYSIS EDGE CASES
# ==========================================================================
echo "--- Section 20: Dimensional Analysis ---" >> "$LOG"
echo "" >> "$LOG"

run_test "DIM" "Multiply m * m = m^2" "2 m * 3 m" "0" ""
run_test "DIM" "Divide m / s = velocity" "10 m / 2 s" "0" ""
run_test "DIM" "F = m*a units" "5 kg * 10 m / (1 s^2)" "0" ""
run_test "DIM" "Energy = F*d" "10 N * 5 m to J" "0" "50"
run_test "DIM" "Power = E/t" "100 J / 10 s to W" "0" "10"
run_test "DIM" "Ohm's law V=IR" "(5 A) * (10 ohm) to V" "0" "50"
run_test "DIM" "Add same dim" "5 m + 3 m" "0" ""
run_test "DIM" "Add different dim" "5 m + 3 s" "1" "mismatch"
run_test "DIM" "Exponent with dim" "(2 m)^3" "0" ""
run_test "DIM" "Dim exponent (error)" "2^(3 m)" "1" "dimensionless"

# ==========================================================================
# SECTION 21: EXTREME / STRESS TESTS
# ==========================================================================
echo "--- Section 21: Stress Tests ---" >> "$LOG"
echo "" >> "$LOG"

run_test "STRESS" "1000-char number" "$(python3 -c 'print("9"*1000)')" "0" ""
run_test "STRESS" "Deep paren nesting (100)" "$(python3 -c 'print("("*100 + "1" + ")"*100)')" "0" "1"
run_test "STRESS" "Long addition chain (200 terms)" "$(python3 -c 'print("+".join(["1"]*200))')" "0" "200"
run_test "STRESS" "Huge factorial-like multiply" "$(python3 -c 'print("*".join([str(i) for i in range(1,21)]))')" "0" ""

# ==========================================================================
# SECTION 22: CONVERSION KEYWORD VARIANTS
# ==========================================================================
echo "--- Section 22: Conversion Keywords ---" >> "$LOG"
echo "" >> "$LOG"

run_test "CONVKEY" "Arrow -> syntax" "5 km -> m" "0" ""
run_test "CONVKEY" "'to' keyword" "5 km to m" "0" ""
run_test "CONVKEY" "'in' keyword" "5 km in m" "0" ""
run_test "CONVKEY" "'as' keyword" "5 km as m" "0" ""

# ==========================================================================
# SECTION 23: SPECIAL NUMBER FORMATS
# ==========================================================================
echo "--- Section 23: Special Numbers ---" >> "$LOG"
echo "" >> "$LOG"

run_test "SPECIAL" "Leading zeros" "007" "0" "7"
run_test "SPECIAL" "Just 0" "0" "0" "0"
run_test "SPECIAL" "Negative zero" "-0" "0" "0"
run_test "SPECIAL" "Very precise float" "3.141592653589793" "0" "3.141592"
run_test "SPECIAL" "Max f64 boundary" "1.7976931348623157e308" "0" ""
run_test "SPECIAL" "Beyond max f64" "1e309" "0" "" # inf?
run_test "SPECIAL" "Smallest positive f64" "5e-324" "0" ""

# ==========================================================================
# SECTION 24: COMBINATORIAL / MULTI-FEATURE
# ==========================================================================
echo "--- Section 24: Combinatorial ---" >> "$LOG"
echo "" >> "$LOG"

run_test "COMBO" "Uncertainty + conversion" "(100±2 degC) to degF" "any" ""
run_test "COMBO" "Percent of unit" "10% of 1 km to m" "0" ""
run_test "COMBO" "Hex to binary" "0xFF to bin" "0" ""
run_test "COMBO" "Expression then to hex" "(200 + 55) to hex" "0" "0xff"
run_test "COMBO" "Function + unit" "sqrt(4 m^2)" "any" ""
run_test "COMBO" "Chained convert attempt" "1 km to m to cm" "any" ""

# ==========================================================================
# SECTION 25: UNIT REGISTRY EDGE CASES
# ==========================================================================
echo "--- Section 25: Unit Registry ---" >> "$LOG"
echo "" >> "$LOG"

run_test "UREG" "Same unit conversion" "1 m to m" "0" "1"
run_test "UREG" "Very small conversion" "1 nm to m" "0" ""
run_test "UREG" "Huge conversion ly->m" "1 ly to m" "0" ""
run_test "UREG" "Data GiB vs GB" "1 GiB to MB" "0" ""
run_test "UREG" "Hz -> rpm" "1 Hz to rpm" "0" "60"
run_test "UREG" "eV (tiny energy)" "1 eV to J" "0" ""

# ==========================================================================
# SUMMARY
# ==========================================================================
echo "============================================================================" >> "$LOG"
echo "SUMMARY" >> "$LOG"
echo "============================================================================" >> "$LOG"
echo "Total Tests: $TOTAL" >> "$LOG"
echo "Passed:      $PASS_COUNT" >> "$LOG"
echo "Failed:      $FAIL_COUNT" >> "$LOG"
echo "Crashed:     $CRASH_COUNT" >> "$LOG"
echo "Pass Rate:   $(echo "scale=1; $PASS_COUNT * 100 / $TOTAL" | bc)%" >> "$LOG"
echo "============================================================================" >> "$LOG"

echo "Done. Total=$TOTAL Pass=$PASS_COUNT Fail=$FAIL_COUNT Crash=$CRASH_COUNT"
echo "Full log: /home/sagnik/code/omnicalc/test_results.log"
