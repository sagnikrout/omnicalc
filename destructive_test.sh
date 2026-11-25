#!/usr/bin/env bash
# ==============================================================================
# OMNICALC UNIFIED MASTER DESTRUCTIVE TEST SUITE
# Comprehensive Edge Cases, Boundary Probes & Calculator Ecosystem Tests
# Target Binary: Resolved dynamically via OMNICALC_BIN / target / PATH
# ==============================================================================

set -u

if [ -z "${OMNICALC_BIN:-}" ]; then
    if [ -x "./target/release/omnicalc" ]; then
        BINARY="./target/release/omnicalc"
    elif [ -x "./target/debug/omnicalc" ]; then
        BINARY="./target/debug/omnicalc"
    elif command -v omnicalc >/dev/null 2>&1; then
        BINARY="$(command -v omnicalc)"
    elif [ -x "$HOME/bin/omnicalc" ]; then
        BINARY="$HOME/bin/omnicalc"
    else
        BINARY="./target/release/omnicalc"
    fi
else
    BINARY="$OMNICALC_BIN"
fi
LOG_FILE="${OMNICALC_LOG:-/tmp/omnicalc_destructive_test.log}"
LOG="$LOG_FILE"
CRASH_LOG="/tmp/omnicalc_crash_repro.log"

rm -f "$LOG_FILE" "$CRASH_LOG"
touch "$LOG_FILE" "$CRASH_LOG"

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
CRASHED_TESTS=0

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

echo "================================================================================"
echo "Starting omnicalc Unified Master Destructive Test Suite"
echo "Target Binary: $BINARY"
echo "Timestamp: $(date -Iseconds)"
echo "================================================================================"

log_entry() {
    local test_id="$1"
    local category="$2"
    local description="$3"
    local cmd="$4"
    local exit_code="$5"
    local stdout="$6"
    local stderr="$7"
    local status="$8"
    local notes="$9"

    printf "[%s] [%s] %s | Status: %s | Exit: %d\n" "$test_id" "$category" "$description" "$status" "$exit_code" >> "$LOG_FILE"
    printf "  Command: %s\n" "$cmd" >> "$LOG_FILE"
    if [ -n "$stdout" ]; then
        printf "  Stdout: %s\n" "$stdout" >> "$LOG_FILE"
    fi
    if [ -n "$stderr" ]; then
        printf "  Stderr: %s\n" "$stderr" >> "$LOG_FILE"
    fi
    if [ -n "$notes" ]; then
        printf "  Notes: %s\n" "$notes" >> "$LOG_FILE"
    fi
    printf -- "--------------------------------------------------------------------------------\n" >> "$LOG_FILE"
}

run_test() {
    local test_id="$1"
    local category="$2"
    local description="$3"
    local expectation="$4"
    shift 4

    local pipe_in=""
    if [ "${1:-}" = "--pipe" ]; then
        pipe_in="$2"
        shift 2
    fi

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    local stdout_file
    local stderr_file
    stdout_file=$(mktemp)
    stderr_file=$(mktemp)

    local exit_code=0
    if [ -n "$pipe_in" ]; then
        printf "%s\n" "$pipe_in" | timeout 2s "$BINARY" "$@" > "$stdout_file" 2> "$stderr_file" || exit_code=$?
    else
        timeout 2s "$BINARY" "$@" > "$stdout_file" 2> "$stderr_file" || exit_code=$?
    fi

    local stdout
    local stderr
    stdout=$(cat "$stdout_file" | tr '\n' ' ' | sed 's/[[:space:]]*$//')
    stderr=$(cat "$stderr_file" | tr '\n' ' ' | sed 's/[[:space:]]*$//')
    rm -f "$stdout_file" "$stderr_file"

    local status="FAIL"
    local notes=""

    if [ "$exit_code" -eq 101 ] || [ "$exit_code" -eq 134 ] || [ "$exit_code" -eq 139 ] || [ "$exit_code" -eq 124 ]; then
        CRASHED_TESTS=$((CRASHED_TESTS + 1))
        status="CRASH"
        case "$exit_code" in
            101) notes="Rust panic detected!" ;;
            134) notes="Fatal runtime stack overflow / abort (SIGABRT)!" ;;
            139) notes="Segmentation fault (SIGSEGV)!" ;;
            124) notes="Process hang / timeout (>2s)!" ;;
        esac
        echo -e "${RED}[CRASH]${NC} $test_id: $description ($notes)"
        echo "CRASH: $test_id | $description | Args: $* | Exit: $exit_code | Err: $stderr" >> "$CRASH_LOG"
        log_entry "$test_id" "$category" "$description" "$BINARY $*" "$exit_code" "$stdout" "$stderr" "$status" "$notes"
        return
    fi

    local exp_type="${expectation%%:*}"
    local exp_val="${expectation#*:}"

    case "$exp_type" in
        "SUCCESS")
            if [ "$exit_code" -eq 0 ]; then
                if [ -z "$exp_val" ] || [[ "$stdout" == *"$exp_val"* ]]; then
                    status="PASS"
                    PASSED_TESTS=$((PASSED_TESTS + 1))
                    echo -e "${GREEN}[PASS]${NC} $test_id: $description"
                else
                    status="FAIL"
                    FAILED_TESTS=$((FAILED_TESTS + 1))
                    notes="Expected stdout substring '$exp_val', got '$stdout'"
                    echo -e "${RED}[FAIL]${NC} $test_id: $description ($notes)"
                fi
            else
                status="FAIL"
                FAILED_TESTS=$((FAILED_TESTS + 1))
                notes="Expected exit 0, got $exit_code. Stderr: $stderr"
                echo -e "${RED}[FAIL]${NC} $test_id: $description ($notes)"
            fi
            ;;
        "ERROR")
            if [ "$exit_code" -ne 0 ]; then
                if [ -z "$exp_val" ] || [[ "$stderr" == *"$exp_val"* ]]; then
                    status="PASS"
                    PASSED_TESTS=$((PASSED_TESTS + 1))
                    echo -e "${GREEN}[PASS]${NC} $test_id: $description"
                else
                    status="FAIL"
                    FAILED_TESTS=$((FAILED_TESTS + 1))
                    notes="Expected stderr substring '$exp_val', got '$stderr'"
                    echo -e "${RED}[FAIL]${NC} $test_id: $description ($notes)"
                fi
            else
                status="FAIL"
                FAILED_TESTS=$((FAILED_TESTS + 1))
                notes="Expected error exit code, got 0. Stdout: $stdout"
                echo -e "${RED}[FAIL]${NC} $test_id: $description ($notes)"
            fi
            ;;
        "CRASH_PROBE")
            if [ "$exit_code" -eq 0 ] || [ "$exit_code" -eq 1 ]; then
                status="PASS"
                PASSED_TESTS=$((PASSED_TESTS + 1))
                echo -e "${GREEN}[PASS]${NC} $test_id: $description"
            else
                status="FAIL"
                FAILED_TESTS=$((FAILED_TESTS + 1))
                notes="Probe returned non-standard exit code $exit_code"
                echo -e "${RED}[FAIL]${NC} $test_id: $description ($notes)"
            fi
            ;;
    esac

    log_entry "$test_id" "$category" "$description" "$BINARY $*" "$exit_code" "$stdout" "$stderr" "$status" "$notes"
}

run_piped_test() {
    local test_id="$1"
    local category="$2"
    local description="$3"
    local gen_cmd="$4"
    local expectation="$5"
    shift 5

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    local stdout_file
    local stderr_file
    stdout_file=$(mktemp)
    stderr_file=$(mktemp)

    local exit_code=0
    eval "$gen_cmd" | timeout 2s "$BINARY" "$@" > "$stdout_file" 2> "$stderr_file" || exit_code=$?

    local stdout
    local stderr
    stdout=$(cat "$stdout_file" | tr '\n' ' ' | sed 's/[[:space:]]*$//')
    stderr=$(cat "$stderr_file" | tr '\n' ' ' | sed 's/[[:space:]]*$//')
    rm -f "$stdout_file" "$stderr_file"

    local status="FAIL"
    local notes=""

    if [ "$exit_code" -eq 101 ] || [ "$exit_code" -eq 134 ] || [ "$exit_code" -eq 139 ] || [ "$exit_code" -eq 124 ]; then
        CRASHED_TESTS=$((CRASHED_TESTS + 1))
        status="CRASH"
        case "$exit_code" in
            101) notes="Rust panic detected!" ;;
            134) notes="Fatal runtime stack overflow / abort (SIGABRT)!" ;;
            139) notes="Segmentation fault (SIGSEGV)!" ;;
            124) notes="Process hang / timeout (>2s)!" ;;
        esac
        echo -e "${RED}[CRASH]${NC} $test_id: $description ($notes)"
        echo "CRASH: $test_id | $description | Piped: $gen_cmd | Exit: $exit_code | Err: $stderr" >> "$CRASH_LOG"
        log_entry "$test_id" "$category" "$description" "$gen_cmd | $BINARY $*" "$exit_code" "$stdout" "$stderr" "$status" "$notes"
        return
    fi

    local exp_type="${expectation%%:*}"
    local exp_val="${expectation#*:}"

    case "$exp_type" in
        "SUCCESS")
            if [ "$exit_code" -eq 0 ] && [[ "$stdout" == *"$exp_val"* ]]; then
                status="PASS"
                PASSED_TESTS=$((PASSED_TESTS + 1))
                echo -e "${GREEN}[PASS]${NC} $test_id: $description"
            else
                status="FAIL"
                FAILED_TESTS=$((FAILED_TESTS + 1))
                notes="Expected stdout '$exp_val', got code $exit_code, stdout: '$stdout'"
                echo -e "${RED}[FAIL]${NC} $test_id: $description ($notes)"
            fi
            ;;
        "ERROR")
            if [ "$exit_code" -ne 0 ] && [[ "$stderr" == *"$exp_val"* ]]; then
                status="PASS"
                PASSED_TESTS=$((PASSED_TESTS + 1))
                echo -e "${GREEN}[PASS]${NC} $test_id: $description"
            else
                status="FAIL"
                FAILED_TESTS=$((FAILED_TESTS + 1))
                notes="Expected stderr '$exp_val', got code $exit_code, stderr: '$stderr'"
                echo -e "${RED}[FAIL]${NC} $test_id: $description ($notes)"
            fi
            ;;
    esac

    log_entry "$test_id" "$category" "$description" "$gen_cmd | $BINARY $*" "$exit_code" "$stdout" "$stderr" "$status" "$notes"
}

run_test_v1() {
    local section="$1"
    local desc="$2"
    local input="$3"
    local expect_status="$4"
    local expect_contains="${5:-}"
    local raw_flag="${6:-}"

    local test_num=$(printf "T%03d" "$((TOTAL_TESTS + 1))")
    local exp="SUCCESS:$expect_contains"
    if [ "$expect_status" = "1" ]; then
        exp="ERROR:$expect_contains"
    elif [ "$expect_status" = "any" ]; then
        exp="CRASH_PROBE"
    fi

    if [ -n "$raw_flag" ]; then
        if [ -n "$input" ]; then
            run_test "$test_num" "$section" "$desc" "$exp" $raw_flag "$input"
        else
            run_test "$test_num" "$section" "$desc" "$exp" $raw_flag
        fi
    else
        run_test "$test_num" "$section" "$desc" "$exp" "$input"
    fi
}

run_pipe_test_v1() {
    local section="$1"
    local desc="$2"
    local pipe_input="$3"
    local expect_status="$4"
    local expect_contains="${5:-}"
    local extra_flags="${6:-}"

    local test_num=$(printf "T%03d" "$((TOTAL_TESTS + 1))")
    local exp="SUCCESS:$expect_contains"
    if [ "$expect_status" = "1" ]; then
        exp="ERROR:$expect_contains"
    elif [ "$expect_status" = "any" ]; then
        exp="CRASH_PROBE"
    fi

    local gen="printf '%s\\n' $(printf '%q' "$pipe_input")"
    if [ -n "$extra_flags" ]; then
        run_piped_test "$test_num" "$section" "$desc" "$gen" "$exp" $extra_flags
    else
        run_piped_test "$test_num" "$section" "$desc" "$gen" "$exp"
    fi
}

echo -e "\n${CYAN}================================================================================${NC}"
echo -e "${CYAN}PART 1: COMPREHENSIVE SUBSYSTEM TEST BATTERY (T001 to T270)${NC}"
echo -e "${CYAN}================================================================================${NC}"
# SECTION 1: EMPTY / DEGENERATE INPUT
# ==========================================================================
echo "--- Section 1: Empty & Degenerate Input ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "EMPTY" "Completely empty string" "" "1" ""
run_test_v1 "EMPTY" "Whitespace only" "   " "1" ""
run_test_v1 "EMPTY" "Single newline" $'\n' "any" ""
run_test_v1 "EMPTY" "Tab characters only" $'\t\t' "any" ""

# ==========================================================================
# SECTION 2: BASIC ARITHMETIC (EXACT RATIONAL)
# ==========================================================================
echo "--- Section 2: Basic Arithmetic ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "ARITH" "Simple addition" "2 + 3" "0" "5"
run_test_v1 "ARITH" "Simple subtraction" "10 - 7" "0" "3"
run_test_v1 "ARITH" "Simple multiplication" "6 * 7" "0" "42"
run_test_v1 "ARITH" "Simple division exact" "10 / 2" "0" "5"
run_test_v1 "ARITH" "Rational result" "1 / 3" "0" "1/3"
run_test_v1 "ARITH" "Negative number" "-5" "0" "-5"
run_test_v1 "ARITH" "Double negative" "--5" "0" "5"
run_test_v1 "ARITH" "Subtraction yielding negative" "3 - 10" "0" "-7"
run_test_v1 "ARITH" "Zero" "0" "0" "0"
run_test_v1 "ARITH" "Modulus" "10 % 3" "0" "1"
run_test_v1 "ARITH" "Modulus zero divisor" "10 % 0" "1" "Division by zero"
run_test_v1 "ARITH" "Power" "2^10" "0" "1024"
run_test_v1 "ARITH" "Power zero" "5^0" "0" "1"
run_test_v1 "ARITH" "Power of zero" "0^5" "0" "0"
run_test_v1 "ARITH" "Zero to zero (0^0)" "0^0" "0" "1"
run_test_v1 "ARITH" "Negative power" "2^-3" "0" "1/8"
run_test_v1 "ARITH" "Chained operations" "2 + 3 * 4" "0" "14"
run_test_v1 "ARITH" "Left paren grouping" "(2 + 3) * 4" "0" "20"
run_test_v1 "ARITH" "Nested parens" "((2 + 3) * (4 + 5))" "0" "45"
run_test_v1 "ARITH" "Deeply nested parens" "((((((1))))))" "0" "1"
run_test_v1 "ARITH" "Unbalanced left paren" "(2 + 3" "1" ""
run_test_v1 "ARITH" "Unbalanced right paren" "2 + 3)" "1" ""
run_test_v1 "ARITH" "Division by zero" "1 / 0" "1" ""
run_test_v1 "ARITH" "Large integer" "999999999999999999 + 1" "0" "1000000000000000000"
run_test_v1 "ARITH" "Very large power" "2^64" "0" "18446744073709551616"
run_test_v1 "ARITH" "Huge power (potential hang)" "2^1000" "0" ""
run_test_v1 "ARITH" "Negative * negative" "-3 * -4" "0" "12"
run_test_v1 "ARITH" "Double star power" "2**10" "0" "1024"

# ==========================================================================
# SECTION 3: FLOATING POINT EDGE CASES
# ==========================================================================
echo "--- Section 3: Floating Point ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "FLOAT" "Basic float" "1.5 + 2.5" "0" "4"
run_test_v1 "FLOAT" "Very small float" "1e-300 + 1e-300" "0" ""
run_test_v1 "FLOAT" "Very large float" "1e308" "0" ""
run_test_v1 "FLOAT" "Overflow float" "1e308 * 10" "0" "" # inf?
run_test_v1 "FLOAT" "Underflow float" "1e-308 / 1e308" "0" ""
run_test_v1 "FLOAT" "Scientific notation" "6.022e23" "0" ""
run_test_v1 "FLOAT" "Negative exponent" "1.6e-19" "0" ""
run_test_v1 "FLOAT" "Float precision 0.1+0.2" "0.1 + 0.2" "0" ""
run_test_v1 "FLOAT" "Leading decimal point" ".5 + .5" "any" ""
run_test_v1 "FLOAT" "Number with underscores" "1_000_000" "0" "1000000"
run_test_v1 "FLOAT" "Only underscore in number" "1__2" "0" "12"

# ==========================================================================
# SECTION 4: BASE CONVERSIONS
# ==========================================================================
echo "--- Section 4: Base Conversions ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "BASE" "Hex input" "0xFF" "0" "255"
run_test_v1 "BASE" "Binary input" "0b1010" "0" "10"
run_test_v1 "BASE" "Octal input" "0o77" "0" "63"
run_test_v1 "BASE" "To hex" "255 to hex" "0" "0xff"
run_test_v1 "BASE" "To bin" "10 to bin" "0" "0b1010"
run_test_v1 "BASE" "To oct" "63 to oct" "0" "0o77"
run_test_v1 "BASE" "To dec (identity)" "42 to dec" "0" "42"
run_test_v1 "BASE" "Hex arithmetic" "0xFF + 1" "0" "256"
run_test_v1 "BASE" "Binary arithmetic" "0b1010 + 0b0101" "0" "15"
run_test_v1 "BASE" "Empty hex 0x" "0x" "any" ""
run_test_v1 "BASE" "Empty binary 0b" "0b" "any" ""
run_test_v1 "BASE" "Empty octal 0o" "0o" "any" ""
run_test_v1 "BASE" "Hex with underscores" "0xFF_FF" "0" "65535"
run_test_v1 "BASE" "Negative to hex" "-1 to hex" "0" ""
run_test_v1 "BASE" "Global base flag hex" "255" "0" "0xff" "-b hex"
run_test_v1 "BASE" "Global base flag bin" "10" "0" "0b1010" "-b bin"

# ==========================================================================
# SECTION 5: BITWISE OPERATIONS
# ==========================================================================
echo "--- Section 5: Bitwise ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "BITWISE" "AND" "0xFF & 0x0F" "0" "15"
run_test_v1 "BITWISE" "OR" "0xF0 | 0x0F" "0" "255"
run_test_v1 "BITWISE" "XOR via caret" "0xFF ^ 0x0F" "0" "" # conflicts with power?
run_test_v1 "BITWISE" "Left shift" "1 << 10" "0" "1024"
run_test_v1 "BITWISE" "Right shift" "1024 >> 5" "0" "32"
run_test_v1 "BITWISE" "NOT (tilde)" "~0" "0" "-1"
run_test_v1 "BITWISE" "NOT 255" "~255" "0" "-256"
run_test_v1 "BITWISE" "Shift by 0" "42 << 0" "0" "42"
run_test_v1 "BITWISE" "Shift by 63" "1 << 63" "0" ""
run_test_v1 "BITWISE" "Large shift (overflow)" "1 << 64" "any" ""  # May panic

# ==========================================================================
# SECTION 6: UNIT CONVERSIONS
# ==========================================================================
echo "--- Section 6: Unit Conversions ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "UNITS" "Meters to feet" "1 m to ft" "0" ""
run_test_v1 "UNITS" "Feet to meters" "1 ft to m" "0" ""
run_test_v1 "UNITS" "Km to miles" "1 km to mi" "0" ""
run_test_v1 "UNITS" "Celsius to Fahrenheit" "100 degC to degF" "0" "212"
run_test_v1 "UNITS" "Fahrenheit to Celsius" "32 degF to degC" "0" "0"
run_test_v1 "UNITS" "Celsius zero" "0 degC to K" "0" "273.15"
run_test_v1 "UNITS" "Absolute zero" "-273.15 degC to K" "0" "0"
run_test_v1 "UNITS" "Kg to pounds" "1 kg to lb" "0" ""
run_test_v1 "UNITS" "Joules to calories" "1 J to cal" "0" ""
run_test_v1 "UNITS" "KWh to joules" "1 kWh to J" "0" "3600000"
run_test_v1 "UNITS" "Watts to horsepower" "746 W to hp" "0" ""
run_test_v1 "UNITS" "Atm to pascal" "1 atm to Pa" "0" "101325"
run_test_v1 "UNITS" "Data: MB to GiB" "1024 MB to GiB" "0" ""
run_test_v1 "UNITS" "Data: megabit" "1 megabit to bit" "0" "1000000"
run_test_v1 "UNITS" "Lightspeed constant c" "1 ly to km" "0" ""
run_test_v1 "UNITS" "Speed: kph to mph" "100 kph to mph" "0" ""
run_test_v1 "UNITS" "Feet-inches to cm" "5'10\" to cm" "0" "177.8"
run_test_v1 "UNITS" "Currency EUR to USD" "1 EUR to USD" "0" "1.038"
run_test_v1 "UNITS" "Currency INR to USD" "85.5 INR to USD" "0" ""
run_test_v1 "UNITS" "Unknown unit" "1 flurb to m" "1" "Unknown"
run_test_v1 "UNITS" "Dimensional mismatch" "1 m to s" "1" "mismatch"
run_test_v1 "UNITS" "Dimensional mismatch add" "5 m + 3 kg" "1" "mismatch"
run_test_v1 "UNITS" "Zero conversion" "0 km to m" "0" "0"
run_test_v1 "UNITS" "Negative temperature" "-40 degC to degF" "0" "-40"
run_test_v1 "UNITS" "Data rate bandwidth" "1 Mbps * 60 s to megabit" "0" "60"
run_test_v1 "UNITS" "Compound: speed * time" "60 kph * 2 hr to km" "0" ""

# ==========================================================================
# SECTION 7: UNCERTAINTY PROPAGATION
# ==========================================================================
echo "--- Section 7: Uncertainty ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "UNCERT" "Simple uncertainty" "5±0.1" "0" "5.0000±0.1000"
run_test_v1 "UNCERT" "Uncertainty addition" "(5±0.1) + (3±0.2)" "0" ""
run_test_v1 "UNCERT" "Uncertainty multiplication" "(5±0.1) * (3±0.2)" "0" ""
run_test_v1 "UNCERT" "Uncertainty division" "(10±0.5) / (2±0.1)" "0" ""
run_test_v1 "UNCERT" "Zero uncertainty" "5±0" "0" ""
run_test_v1 "UNCERT" "Negative nominal" "-5±0.1" "0" ""
run_test_v1 "UNCERT" "Uncertainty with units" "(2±0.02 J) / (523±5 W) to ms" "0" ""
run_test_v1 "UNCERT" "Uncertainty +/- syntax" "10+/-0.5" "0" ""
run_test_v1 "UNCERT" "Zero nominal with error" "0±0.5" "0" ""
run_test_v1 "UNCERT" "Very large error" "1±1e10" "0" ""

# ==========================================================================
# SECTION 8: SYMBOLIC DIFFERENTIATION
# ==========================================================================
echo "--- Section 8: Symbolic Differentiation ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "DIFF" "Linear" "diff(3x)" "0" "3"
run_test_v1 "DIFF" "Quadratic" "diff(x^2)" "0" "2x"
run_test_v1 "DIFF" "Cubic" "diff(x^3)" "0" "3 * x ^ 2"
run_test_v1 "DIFF" "Polynomial" "diff(6x^2)" "0" "12x"
run_test_v1 "DIFF" "Constant" "diff(42)" "0" "0"
run_test_v1 "DIFF" "sin" "diff(sin(x))" "0" "cos"
run_test_v1 "DIFF" "cos" "diff(cos(x))" "0" "sin"
run_test_v1 "DIFF" "exp" "diff(exp(x))" "0" "exp"
run_test_v1 "DIFF" "ln" "diff(ln(x))" "0" ""
run_test_v1 "DIFF" "Sum" "diff(x^2 + 3x + 1)" "0" ""
run_test_v1 "DIFF" "Product" "diff(x * sin(x))" "0" ""
run_test_v1 "DIFF" "Quotient" "diff(x / sin(x))" "0" ""
run_test_v1 "DIFF" "Chain rule" "diff(sin(x^2))" "0" ""
run_test_v1 "DIFF" "No variable (all constant)" "diff(5 + 3)" "0" "0"

# ==========================================================================
# SECTION 9: NUMERICAL INTEGRATION
# ==========================================================================
echo "--- Section 9: Integration ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "INTEG" "Polynomial" "integrate(x^2; 0; 1)" "0" ""
run_test_v1 "INTEG" "Known integral 6x^2" "integrate(6x^2; 1; 5)" "0" "248"
run_test_v1 "INTEG" "Sin from 0 to pi" "integrate(sin(x); 0; 3.14159265)" "0" ""
run_test_v1 "INTEG" "Constant function" "integrate(1; 0; 10)" "0" "10"
run_test_v1 "INTEG" "Negative bounds" "integrate(x; -5; 5)" "0" "0"
run_test_v1 "INTEG" "Reversed bounds" "integrate(x^2; 5; 1)" "0" ""
run_test_v1 "INTEG" "Zero-width interval" "integrate(x; 3; 3)" "0" "0"
run_test_v1 "INTEG" "1 arg only (error)" "integrate(x)" "1" ""
run_test_v1 "INTEG" "2 args only (error)" "integrate(x; 0)" "1" ""
run_test_v1 "INTEG" "With explicit var" "integrate(x^2; x; 0; 1)" "0" ""

# ==========================================================================
# SECTION 10: EQUATION SOLVING
# ==========================================================================
echo "--- Section 10: Equation Solving ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "SOLVE" "Linear 2x=10" "2x = 10" "0" "x = 5"
run_test_v1 "SOLVE" "Linear 2x+4=10" "2x + 4 = 10" "0" "x = 3"
run_test_v1 "SOLVE" "Linear x-7=0" "x - 7 = 0" "0" "x = 7"
run_test_v1 "SOLVE" "Zero LHS" "0 = x" "0" ""
run_test_v1 "SOLVE" "Identity 0=0" "0 = 0" "any" ""
run_test_v1 "SOLVE" "Contradiction 1=2" "1 = 2" "any" ""
run_test_v1 "SOLVE" "Quadratic x^2=4" "x^2 = 4" "0" ""

# ==========================================================================
# SECTION 11: FACTORING
# ==========================================================================
echo "--- Section 11: Factoring ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "FACTOR" "Difference of squares" "factor(x^2 - 9)" "0" "(x - 3)(x + 3)"
run_test_v1 "FACTOR" "Perfect square" "factor(x^2 - 6x + 9)" "0" ""
run_test_v1 "FACTOR" "No real roots" "factor(x^2 + 1)" "0" ""
run_test_v1 "FACTOR" "Linear" "factor(2x + 4)" "0" ""
run_test_v1 "FACTOR" "Constant" "factor(42)" "0" ""

# ==========================================================================
# SECTION 12: MATH FUNCTIONS
# ==========================================================================
echo "--- Section 12: Math Functions ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "FUNC" "sqrt(4)" "sqrt(4)" "0" "2"
run_test_v1 "FUNC" "sqrt(2)" "sqrt(2)" "0" "1.414"
run_test_v1 "FUNC" "sqrt(-1) => complex" "sqrt(-1)" "0" "i"
run_test_v1 "FUNC" "sqrt(0)" "sqrt(0)" "0" "0"
run_test_v1 "FUNC" "cbrt(27)" "cbrt(27)" "0" "3"
run_test_v1 "FUNC" "cbrt(-8)" "cbrt(-8)" "0" "-2"
run_test_v1 "FUNC" "sin(0)" "sin(0)" "0" "0"
run_test_v1 "FUNC" "cos(0)" "cos(0)" "0" "1"
run_test_v1 "FUNC" "tan(0)" "tan(0)" "0" "0"
run_test_v1 "FUNC" "sin(pi)" "sin(pi)" "0" ""
run_test_v1 "FUNC" "cos(pi)" "cos(pi)" "0" "-1"
run_test_v1 "FUNC" "asin(1)" "asin(1)" "0" ""
run_test_v1 "FUNC" "acos(0)" "acos(0)" "0" ""
run_test_v1 "FUNC" "asin(2) domain error" "asin(2)" "1" "Domain error"
run_test_v1 "FUNC" "exp(0)" "exp(0)" "0" "1"
run_test_v1 "FUNC" "exp(1)" "exp(1)" "0" "2.718"
run_test_v1 "FUNC" "ln(1)" "ln(1)" "0" "0"
run_test_v1 "FUNC" "ln(e)" "ln(e)" "0" "1"
run_test_v1 "FUNC" "ln(0)" "ln(0)" "0" "" # -inf or complex
run_test_v1 "FUNC" "ln(-1)" "ln(-1)" "0" "" # complex
run_test_v1 "FUNC" "log10(100)" "log10(100)" "0" "2"
run_test_v1 "FUNC" "log10(0)" "log10(0)" "0" "" # -inf
run_test_v1 "FUNC" "abs(-5)" "abs(-5)" "0" "5"
run_test_v1 "FUNC" "abs(5)" "abs(5)" "0" "5"
run_test_v1 "FUNC" "round(2.7)" "round(2.7)" "0" "3"
run_test_v1 "FUNC" "floor(2.9)" "floor(2.9)" "0" "2"
run_test_v1 "FUNC" "ceil(2.1)" "ceil(2.1)" "0" "3"
run_test_v1 "FUNC" "normdist(0)" "normdist(0)" "0" "0.5"
run_test_v1 "FUNC" "No-arg function" "sin()" "1" "requires"
run_test_v1 "FUNC" "Unknown function" "bogus(5)" "1" "Unknown"

# ==========================================================================
# SECTION 13: COMPLEX NUMBERS
# ==========================================================================
echo "--- Section 13: Complex Numbers ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "COMPLEX" "i literal" "i" "0" ""
run_test_v1 "COMPLEX" "i squared" "i * i" "0" "" # should be -1
run_test_v1 "COMPLEX" "i^2" "i^2" "0" "" # -1
run_test_v1 "COMPLEX" "Complex addition" "(3 + 2i) + (1 + 4i)" "any" ""  # may not parse
run_test_v1 "COMPLEX" "sqrt(-4)" "sqrt(-4)" "0" "2"

# ==========================================================================
# SECTION 14: CONSTANTS
# ==========================================================================
echo "--- Section 14: Constants ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "CONST" "pi value" "pi" "0" "3.14159"
run_test_v1 "CONST" "tau value" "tau" "0" "6.28318"
run_test_v1 "CONST" "e value" "e" "0" "2.71828"
run_test_v1 "CONST" "pi Unicode π" "π" "0" "3.14159"
run_test_v1 "CONST" "tau Unicode τ" "τ" "0" "6.28318"
run_test_v1 "CONST" "pi/2" "pi / 2" "0" "1.5707"

# ==========================================================================
# SECTION 15: PERCENT & OF
# ==========================================================================
echo "--- Section 15: Percent & Of ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "PERCENT" "50 percent" "50%" "0" "1/2"
run_test_v1 "PERCENT" "5% of 200" "5% of 200" "0" "10"
run_test_v1 "PERCENT" "100% of 50" "100% of 50" "0" "50"
run_test_v1 "PERCENT" "0% of 100" "0% of 100" "0" "0"
run_test_v1 "PERCENT" "150% of 100" "150% of 100" "0" "150"
run_test_v1 "PERCENT" "Decimal percent 12.5% of 80" "12.5% of 80" "0" "10"

# ==========================================================================
# SECTION 16: IMPLICIT MULTIPLICATION
# ==========================================================================
echo "--- Section 16: Implicit Multiplication ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "IMPL_MUL" "2pi" "2pi" "0" "6.28318"
run_test_v1 "IMPL_MUL" "3(4+5)" "3(4 + 5)" "0" "27"
run_test_v1 "IMPL_MUL" "Number unit: 5 m" "5 m to cm" "0" "500"
run_test_v1 "IMPL_MUL" "Adjacent parens (2)(3)" "(2)(3)" "0" "6"
run_test_v1 "IMPL_MUL" "2sin(0)" "2sin(0)" "0" "0"
run_test_v1 "IMPL_MUL" "Ambiguous: 2x where x unknown" "2x" "0" "2x"

# ==========================================================================
# SECTION 17: PARSER EDGE CASES / MALFORMED INPUT
# ==========================================================================
echo "--- Section 17: Parser Edge Cases ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "PARSE" "Only operator +" "+" "1" ""
run_test_v1 "PARSE" "Only operator *" "*" "1" ""
run_test_v1 "PARSE" "Only operator /" "/" "1" ""
run_test_v1 "PARSE" "Double operator ++" "5 ++ 3" "any" ""
run_test_v1 "PARSE" "Trailing operator" "5 +" "1" ""
run_test_v1 "PARSE" "Leading operator *" "* 5" "1" ""
run_test_v1 "PARSE" "Multiple dots" "1.2.3" "any" ""
run_test_v1 "PARSE" "Only dot" "." "1" ""
run_test_v1 "PARSE" "Exclamation mark" "5!" "1" ""
run_test_v1 "PARSE" "At sign" "@" "1" ""
run_test_v1 "PARSE" "Hash" "#" "1" ""
run_test_v1 "PARSE" "Dollar sign" "\$5" "1" ""
run_test_v1 "PARSE" "Backslash" "\\" "1" ""
run_test_v1 "PARSE" "Curly braces" "{1+2}" "1" ""
run_test_v1 "PARSE" "Square brackets" "[1+2]" "0" "[3]"
run_test_v1 "PARSE" "Very long expression" "1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1" "0" "50"
run_test_v1 "PARSE" "Unicode emoji" "😀" "1" ""
run_test_v1 "PARSE" "Mixed valid/invalid" "2 + @ + 3" "1" ""
run_test_v1 "PARSE" "Semicolon at top level" "5; 3" "1" ""
run_test_v1 "PARSE" "Multiple equals" "x = y = 5" "any" ""

# ==========================================================================
# SECTION 18: CLI FLAGS & PIPE BEHAVIOR
# ==========================================================================
echo "--- Section 18: CLI Flags & Piping ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "CLI" "Eval flag -e" "2 + 3" "0" "5" "-e"
run_test_v1 "CLI" "Raw flag" "1/3" "0" "0.333333" "--raw"
run_test_v1 "CLI" "Raw with units" "5 m / (3 s)" "0" "1.6666" "--raw"
run_test_v1 "CLI" "Version" "" "0" "2026.11.18" "--version"
run_test_v1 "CLI" "Help" "" "0" "omnicalc" "--help"

run_pipe_test_v1 "PIPE" "Stdin single line" "2 + 3" "0" "5"
run_pipe_test_v1 "PIPE" "Stdin multi-line" "2 + 3
4 * 5" "0" "5"
run_pipe_test_v1 "PIPE" "Stdin with comments" "# comment
2 + 3" "0" "5"
run_pipe_test_v1 "PIPE" "Stdin empty lines" "
2 + 3
" "0" "5"
run_pipe_test_v1 "PIPE" "Stdin error mid-stream" "2 + 3
1 / 0
4 + 5" "1" ""
run_pipe_test_v1 "PIPE" "Stdin with raw flag" "1/3" "0" "0.333333" "--raw"
run_pipe_test_v1 "PIPE" "Stdin empty" "" "0" ""

# ==========================================================================
# SECTION 19: TEMPERATURE CONVERSION CORRECTNESS
# ==========================================================================
echo "--- Section 19: Temperature Conversion Deep ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "TEMP" "Boiling point C->F" "100 degC to degF" "0" "212"
run_test_v1 "TEMP" "Freezing point C->F" "0 degC to degF" "0" "32"
run_test_v1 "TEMP" "Body temp F->C" "98.6 degF to degC" "0" "37"
run_test_v1 "TEMP" "Kelvin to Celsius" "373.15 K to degC" "0" "100"
run_test_v1 "TEMP" "Celsius to Kelvin" "0 degC to K" "0" "273.15"
run_test_v1 "TEMP" "-40 crossover" "-40 degC to degF" "0" "-40"
run_test_v1 "TEMP" "-40 F to C crossover" "-40 degF to degC" "0" "-40"

# ==========================================================================
# SECTION 20: DIMENSIONAL ANALYSIS EDGE CASES
# ==========================================================================
echo "--- Section 20: Dimensional Analysis ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "DIM" "Multiply m * m = m^2" "2 m * 3 m" "0" ""
run_test_v1 "DIM" "Divide m / s = velocity" "10 m / 2 s" "0" ""
run_test_v1 "DIM" "F = m*a units" "5 kg * 10 m / (1 s^2)" "0" ""
run_test_v1 "DIM" "Energy = F*d" "10 N * 5 m to J" "0" "50"
run_test_v1 "DIM" "Power = E/t" "100 J / 10 s to W" "0" "10"
run_test_v1 "DIM" "Ohm's law V=IR" "(5 A) * (10 ohm) to V" "0" "50"
run_test_v1 "DIM" "Add same dim" "5 m + 3 m" "0" ""
run_test_v1 "DIM" "Add different dim" "5 m + 3 s" "1" "mismatch"
run_test_v1 "DIM" "Exponent with dim" "(2 m)^3" "0" ""
run_test_v1 "DIM" "Dim exponent (error)" "2^(3 m)" "1" "dimensionless"

# ==========================================================================
# SECTION 21: EXTREME / STRESS TESTS
# ==========================================================================
echo "--- Section 21: Stress Tests ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "STRESS" "1000-char number" "$(python3 -c 'print("9"*1000)')" "0" ""
run_test_v1 "STRESS" "Deep paren nesting (100)" "$(python3 -c 'print("("*100 + "1" + ")"*100)')" "0" "1"
run_test_v1 "STRESS" "Long addition chain (200 terms)" "$(python3 -c 'print("+".join(["1"]*200))')" "0" "200"
run_test_v1 "STRESS" "Huge factorial-like multiply" "$(python3 -c 'print("*".join([str(i) for i in range(1,21)]))')" "0" ""

# ==========================================================================
# SECTION 22: CONVERSION KEYWORD VARIANTS
# ==========================================================================
echo "--- Section 22: Conversion Keywords ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "CONVKEY" "Arrow -> syntax" "5 km -> m" "0" ""
run_test_v1 "CONVKEY" "'to' keyword" "5 km to m" "0" ""
run_test_v1 "CONVKEY" "'in' keyword" "5 km in m" "0" ""
run_test_v1 "CONVKEY" "'as' keyword" "5 km as m" "0" ""

# ==========================================================================
# SECTION 23: SPECIAL NUMBER FORMATS
# ==========================================================================
echo "--- Section 23: Special Numbers ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "SPECIAL" "Leading zeros" "007" "0" "7"
run_test_v1 "SPECIAL" "Just 0" "0" "0" "0"
run_test_v1 "SPECIAL" "Negative zero" "-0" "0" "0"
run_test_v1 "SPECIAL" "Very precise float" "3.141592653589793" "0" "3.141592"
run_test_v1 "SPECIAL" "Max f64 boundary" "1.7976931348623157e308" "0" ""
run_test_v1 "SPECIAL" "Beyond max f64" "1e309" "0" "" # inf?
run_test_v1 "SPECIAL" "Smallest positive f64" "5e-324" "0" ""

# ==========================================================================
# SECTION 24: COMBINATORIAL / MULTI-FEATURE
# ==========================================================================
echo "--- Section 24: Combinatorial ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "COMBO" "Uncertainty + conversion" "(100±2 degC) to degF" "any" ""
run_test_v1 "COMBO" "Percent of unit" "10% of 1 km to m" "0" ""
run_test_v1 "COMBO" "Hex to binary" "0xFF to bin" "0" ""
run_test_v1 "COMBO" "Expression then to hex" "(200 + 55) to hex" "0" "0xff"
run_test_v1 "COMBO" "Function + unit" "sqrt(4 m^2)" "any" ""
run_test_v1 "COMBO" "Chained convert attempt" "1 km to m to cm" "any" ""

# ==========================================================================
# SECTION 25: UNIT REGISTRY EDGE CASES
# ==========================================================================
echo "--- Section 25: Unit Registry ---" >> "$LOG"
echo "" >> "$LOG"

run_test_v1 "UREG" "Same unit conversion" "1 m to m" "0" "1"
run_test_v1 "UREG" "Very small conversion" "1 nm to m" "0" ""
run_test_v1 "UREG" "Huge conversion ly->m" "1 ly to m" "0" ""
run_test_v1 "UREG" "Data GiB vs GB" "1 GiB to MB" "0" ""
run_test_v1 "UREG" "Hz -> rpm" "1 Hz to rpm" "0" "60"
run_test_v1 "UREG" "eV (tiny energy)" "1 eV to J" "0" ""


# ------------------------------------------------------------------------------
# CATEGORY 1: ARITHMETIC CRASHES, POWER OVERFLOWS, AND BOUNDARIES
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 1: Arithmetic Crashes, Power Overflows & Boundaries ===${NC}"
run_test "T301" "Arithmetic" "Zero to negative power (0^-1 panic probe)" "ERROR:Division by zero" -e "0^-1"
run_test "T302" "Arithmetic" "Zero to negative two (0^-2 panic probe)" "ERROR:Division by zero" -e "0^-2"
run_test "T303" "Arithmetic" "Zero to power zero (0^0 exact definition)" "SUCCESS:1" -e "0^0"
run_test "T304" "Arithmetic" "Large exponent loop exhaustion (2^10000000 hang probe)" "ERROR:Exponent too large" -e "2^10000000"
run_test "T305" "Arithmetic" "Negative base fractional power (-1)^0.5 complex evaluation" "SUCCESS:i" -e "(-1)^0.5"
run_test "T306" "Arithmetic" "Negative base fractional power (-4)^0.5 complex evaluation" "SUCCESS:2i" -e "(-4)^0.5"
run_test "T307" "Arithmetic" "Negative base cube root (-8)^(1/3) principal complex root" "SUCCESS:1 + 1.732" -e "(-8)^(1/3)"
run_test "T308" "Arithmetic" "Exact rational cube root cbrt(-8)" "SUCCESS:-2" -e "cbrt(-8)"
run_test "T309" "Arithmetic" "Modulo by zero with float 5 % 0.0" "ERROR:Division by zero in modulus" -e "5 % 0.0"
run_test "T310" "Arithmetic" "Exact zero modulo 0 % 5" "SUCCESS:0" -e "0 % 5"

# ------------------------------------------------------------------------------
# CATEGORY 2: STACK OVERFLOW & RECURSION LIMITS
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 2: Stack Overflow & Recursion Limits ===${NC}"
run_piped_test "T311" "Recursion" "Deeply nested parentheses (1000 parens)" "python3 -c \"print('(' * 1000 + '1' + ')' * 1000)\"" "ERROR:Recursion depth limit exceeded"
run_piped_test "T312" "Recursion" "Deeply nested unary pluses (1000 pluses)" "python3 -c \"print('+' * 1000 + '1')\"" "ERROR:Recursion depth limit exceeded"
run_piped_test "T313" "Recursion" "Deeply nested unary minuses (1000 minuses)" "python3 -c \"print('-' * 1000 + '1')\"" "ERROR:Recursion depth limit exceeded"
run_piped_test "T314" "Recursion" "Moderate nested parentheses (50 parens - should pass)" "python3 -c \"print('(' * 50 + '42' + ')' * 50)\"" "SUCCESS:42"

# ------------------------------------------------------------------------------
# CATEGORY 3: COMPLEX NUMBER FUNCTIONS (TRIG, EXP, LOG)
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 3: Complex Number Functions (Euler & Trigonometry) ===${NC}"
run_test "T315" "Complex" "Euler's identity exp(i * pi) should equal -1" "SUCCESS:-1" -e "exp(i * pi)"
run_test "T316" "Complex" "Complex sine sin(i) should equal ~1.1752i" "SUCCESS:1.1752" -e "sin(i)"
run_test "T317" "Complex" "Complex cosine cos(i) should equal ~1.54308" "SUCCESS:1.543" -e "cos(i)"
run_test "T318" "Complex" "Complex tangent tan(i) should equal ~0.76159i" "SUCCESS:0.7615" -e "tan(i)"
run_test "T319" "Complex" "Complex exponential exp(1 + i)" "SUCCESS:1.46869" -e "exp(1 + i)"
run_test "T320" "Complex" "Logarithm of negative number log10(-1)" "SUCCESS:1.3643763538i" -e "log10(-1)"
run_test "T321" "Complex" "Natural log of negative number ln(-1)" "SUCCESS:3.141592653" -e "ln(-1)"
run_test "T322" "Complex" "Imaginary unit powered by itself i^i" "SUCCESS:0.2078795" -e "i^i"
run_test "T323" "Complex" "Complex reciprocal 1 / i" "SUCCESS:-i" -e "1 / i"
run_test "T324" "Complex" "Complex reciprocal 1 / (1 + i)" "SUCCESS:0.5 - 0.5i" -e "1 / (1 + i)"

# ------------------------------------------------------------------------------
# CATEGORY 4: UNCERTAINTY PROPAGATION & BOUNDARY CONDITIONS
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 4: Uncertainty Propagation Edge Cases ===${NC}"
run_test "T325" "Uncertainty" "Multiplication when nominal is zero (0 +/- 1) * (5 +/- 1)" "SUCCESS:0.0000±5.0000" -e "(0 +/- 1) * (5 +/- 1)"
run_test "T326" "Uncertainty" "Division when nominal is zero (0 +/- 1) / (5 +/- 1)" "SUCCESS:0.0000±0.2000" -e "(0 +/- 1) / (5 +/- 1)"
run_test "T327" "Uncertainty" "Product of zero-error and non-zero (10 +/- 0) * (5 +/- 1)" "SUCCESS:50.0000±10.0000" -e "(10 +/- 0) * (5 +/- 1)"
run_test "T328" "Uncertainty" "Square root of uncertainty sqrt(4 +/- 0.4)" "SUCCESS:2.0000±0.1000" -e "sqrt(4 +/- 0.4)"
run_test "T329" "Uncertainty" "Square of uncertainty (2 +/- 0.1)^2" "SUCCESS:4.0000±0.4000" -e "(2 +/- 0.1)^2"
run_test "T330" "Uncertainty" "Precedence of uncertainty with units 10 m +/- 1 m" "SUCCESS:10.0000±1.0000 m" -e "10 m +/- 1 m"

# ------------------------------------------------------------------------------
# CATEGORY 5: LEXER & NUMBER FORMATTING ANOMALIES
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 5: Lexer & Number Formatting Anomalies ===${NC}"
run_test "T331" "Lexer" "Leading decimal dot .5 + .5" "SUCCESS:1" -e ".5 + .5"
run_test "T332" "Lexer" "Leading decimal dot in subtraction .75 - .25" "SUCCESS:0.5" -e ".75 - .25"
run_test "T333" "Lexer" "Trailing decimal dot 5. + 5." "SUCCESS:10" -e "5. + 5."
run_test "T334" "Lexer" "Trailing decimal dot in product 2.5 * 4." "SUCCESS:10" -e "2.5 * 4."
run_test "T335" "Lexer" "Negative leading decimal dot -.5 * 2" "SUCCESS:-1" -e "-.5 * 2"
run_test "T336" "Lexer" "Scientific notation with capital E: 1E6 + 1E3" "SUCCESS:1001000" -e "1E6 + 1E3"
run_test "T337" "Lexer" "Scientific notation with positive exponent 2.5e+3" "SUCCESS:2500" -e "2.5e+3"
run_test "T338" "Lexer" "Incomplete scientific notation 1e * 2" "SUCCESS:5.436" -e "1e * 2"

# ------------------------------------------------------------------------------
# CATEGORY 6: BITWISE ENGINE & LARGE INTEGER LITERALS
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 6: Bitwise Engine & Large Literal Overflow ===${NC}"
run_test "T339" "Bitwise" "64-bit unsigned hex mask 0xffffffffffffffff" "SUCCESS:18446744073709551615" -e "0xffffffffffffffff"
run_test "T340" "Bitwise" "64-bit unsigned hex mask to hex" "SUCCESS:0xffffffffffffffff" -e "0xffffffffffffffff to hex"
run_test "T341" "Bitwise" "128-bit hex literal 0x10000000000000000" "SUCCESS:18446744073709551616" -e "0x10000000000000000"
run_test "T342" "Bitwise" "64-bit binary literal bit 63 set" "SUCCESS:9223372036854775808" -e "0b1000000000000000000000000000000000000000000000000000000000000000"
run_test "T343" "Bitwise" "Negative left shift 1 << -1 (must reject)" "ERROR:Negative shift amount" -e "1 << -1"
run_test "T344" "Bitwise" "Negative right shift 1 >> -5 (must reject)" "ERROR:Negative shift amount" -e "1 >> -5"
run_test "T345" "Bitwise" "Shift amount exceeding 63 bits 1 << 64" "ERROR:exceeds maximum 63" -e "1 << 64"
run_test "T346" "Bitwise" "Bitwise NOT of zero to hex (~0 to hex)" "SUCCESS:0x-1" -e "~0 to hex"

# ------------------------------------------------------------------------------
# CATEGORY 7: MODULUS VS PERCENTAGE DISAMBIGUATION
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 7: Modulus vs Percentage Disambiguation ===${NC}"
run_test "T347" "Percent" "Modulo with variable: 50 % x (must be remainder)" "SUCCESS:50 % x" -e "50 % x"
run_test "T348" "Percent" "Modulo with negative operand 10 % -3" "SUCCESS:1" -e "10 % -3"
run_test "T349" "Percent" "Modulo with hex operand 10 % 0x3" "SUCCESS:1" -e "10 % 0x3"
run_test "T350" "Percent" "Percentage of value: 50% of 200" "SUCCESS:100" -e "50% of 200"
run_test "T351" "Percent" "Simple percentage: 50%" "SUCCESS:1/2" -e "50%"
run_test "T352" "Percent" "Addition of percentage: 100 + 10%" "SUCCESS:1001/10" -e "100 + 10%"

# ------------------------------------------------------------------------------
# CATEGORY 8: UNIT CONVERSION PARSING & INCHES AMBIGUITY
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 8: Unit Conversion Parsing & Inches Ambiguity ===${NC}"
run_test "T353" "Units" "Conversion to inches using 'to in': 5 cm to in" "SUCCESS:1.9685" -e "5 cm to in"
run_test "T354" "Units" "Conversion to inches using 'in in': 5 cm in in" "SUCCESS:1.9685" -e "5 cm in in"
run_test "T355" "Units" "Conversion from inches: 10 in to cm" "SUCCESS:25.4 cm" -e "10 in to cm"
run_test "T356" "Units" "Feet notation standalone: 5' to cm" "SUCCESS:152.4 cm" -e "5' to cm"
run_test "T357" "Units" "Inches notation standalone: 10\" to cm" "SUCCESS:25.4 cm" -e '10" to cm'
run_test "T358" "Units" "Combined feet and inches: 5'10\" to cm" "SUCCESS:177.8 cm" -e '5'\''10" to cm'
run_test "T359" "Units" "Compound target unit with slash: 100 km/h to m/s" "SUCCESS:27.77" -e "100 km/h to m/s"
run_test "T360" "Units" "Compound target unit with power: 100 N / 2 m^2 to Pa" "SUCCESS:50 Pa" -e "100 N / 2 m^2 to Pa"
run_test "T361" "Units" "Unit preservation in single quantity: 5 km" "SUCCESS:5 km" -e "5 km"
run_test "T362" "Units" "Celsius display without conversion: 10 degC" "SUCCESS:10 degC" -e "10 degC"
run_test "T363" "Units" "Fahrenheit display without conversion: 100 degF" "SUCCESS:100 degF" -e "100 degF"
run_test "T364" "Units" "Odd dimensional square root sqrt(4 m) (must reject)" "ERROR:not divisible by 2" -e "sqrt(4 m)"
run_test "T365" "Units" "Valid dimensional square root sqrt(4 m^2)" "SUCCESS:2 m" -e "sqrt(4 m^2)"

# ------------------------------------------------------------------------------
# CATEGORY 9: TRANSCENDENTAL EQUATION SOLVING
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 9: Transcendental Equation Solving ===${NC}"
run_test "T366" "Equations" "Trigonometric equation sin(x) = 0" "SUCCESS:x ≈ 0" -e "sin(x) = 0"
run_test "T367" "Equations" "Trigonometric equation cos(x) = 0" "SUCCESS:x ≈ 1.5707" -e "cos(x) = 0"
run_test "T368" "Equations" "Exponential equation exp(x) = 10" "SUCCESS:x ≈ 2.3025" -e "exp(x) = 10"
run_test "T369" "Equations" "Logarithmic equation ln(x) = 2" "SUCCESS:x ≈ 7.389" -e "ln(x) = 2"
run_test "T370" "Equations" "Square root equation sqrt(x) = 4" "SUCCESS:x ≈ 16" -e "sqrt(x) = 4"
run_test "T371" "Equations" "Polynomial cubic equation x^3 - 27 = 0" "SUCCESS:x ≈ 3" -e "x^3 - 27 = 0"

# ------------------------------------------------------------------------------
# CATEGORY 10: HIGHER-ORDER DERIVATIVES & SYMBOLIC ALGEBRA
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 10: Higher-Order Derivatives & Symbolic Algebra ===${NC}"
run_test "T372" "Symbolic" "Double prime second derivative (x^3)''" "SUCCESS:6x" -e "(x^3)''"
run_test "T373" "Symbolic" "Nested diff second derivative diff(diff(x^3, x), x)" "SUCCESS:6x" -e "diff(diff(x^3, x), x)"
run_test "T374" "Symbolic" "Derivative of tangent diff(tan(x), x)" "SUCCESS:1 / cos(x) ^ 2" -e "diff(tan(x), x)"
run_test "T375" "Symbolic" "Derivative of square root diff(sqrt(x), x)" "SUCCESS:1 / 2 * sqrt(x)" -e "diff(sqrt(x), x)"
run_test "T376" "Symbolic" "Integer prime factorization 12 to factors" "SUCCESS:2^2 * 3" -e "12 to factors"
run_test "T377" "Symbolic" "Prime number factorization 13 to factors" "SUCCESS:13" -e "13 to factors"

# ------------------------------------------------------------------------------
# CATEGORY 11: CLI FLAGS & FORMATTING INTERACTION
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 11: CLI Flags & Formatting Interaction ===${NC}"
run_test "T378" "CLI" "Raw mode with hex base (-r -b hex -e 255)" "SUCCESS:ff" -r -b hex -e "255"
run_test "T379" "CLI" "Raw mode with binary base (-r -b bin -e 10)" "SUCCESS:1010" -r -b bin -e "10"
run_test "T380" "CLI" "Raw mode with octal base (-r -b oct -e 64)" "SUCCESS:100" -r -b oct -e "64"
run_test "T381" "CLI" "Positional expression with negative arguments" "SUCCESS:5" -- "-5 + 10"
run_test "T382" "CLI" "Empty string input exits non-zero" "ERROR:Empty input" -e ""

# ------------------------------------------------------------------------------
# CATEGORY 12: STATISTICAL DISTRIBUTION BOUNDARY GUARDS
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 12: Statistical Distribution Boundary Guards ===${NC}"
run_test "T383" "Statistics" "Standard normal distribution normdist(0)" "SUCCESS:0.5" -e "normdist(0)"
run_test "T384" "Statistics" "Normal distribution with zero std dev normdist(0, 0, 0)" "ERROR:Standard deviation must be positive" -e "normdist(0, 0, 0)"
run_test "T385" "Statistics" "Normal distribution with negative std dev normdist(1, 0, -1)" "ERROR:Standard deviation must be positive" -e "normdist(1, 0, -1)"
run_test "T386" "Statistics" "Normal distribution extreme positive normdist(1e30)" "SUCCESS:1" -e "normdist(1e30)"
run_test "T387" "Statistics" "Normal distribution extreme negative normdist(-1e30)" "SUCCESS:0" -e "normdist(-1e30)"

# ------------------------------------------------------------------------------
# CATEGORY 13: MASTER CROSS-CALCULATOR ECOSYSTEM INTEGRATION (85-TEST CATALOG)
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}=== CATEGORY 13: Master Cross-Calculator Ecosystem Integration (85 Tests) ===${NC}"
run_test "TC-PREC-01" "Precedence" "Unary minus vs power: -2^2" "SUCCESS:-4" -e '-2^2'
run_test "TC-PREC-02" "Precedence" "Grouped power: (-2)^2" "SUCCESS:4" -e '(-2)^2'
run_test "TC-PREC-03" "Precedence" "Right associative power: 2^3^2" "SUCCESS:512" -e '2^3^2'
run_test "TC-PREC-04" "Precedence" "Implicit multiplication binding: 1/2x" "SUCCESS:1 / 2x" -e '1 / 2x'
run_test "TC-PREC-05" "Precedence" "Explicit multiplication: 1/2*x" "SUCCESS:2" -e '1 / 2 * 4'
run_test "TC-PREC-06" "Precedence" "Unit denominator binding: 100 J / 10 s" "SUCCESS:10 W" -e '100 J / 10 s'
run_test "TC-PREC-07" "Precedence" "Explicit unit multiplication: 100 J / 10 * 1 s" "SUCCESS:10" -e '100 J / 10 * 1 s'
run_test "TC-PREC-08" "Precedence" "Factorial function precedence: fact(5) + 3" "SUCCESS:123" -e 'fact(5) + 3'
run_test "TC-PREC-09" "Precedence" "Bitwise shift precedence: 1 << 2 + 3" "SUCCESS:32" -e '1 << 2 + 3'
run_test "TC-PREC-10" "Precedence" "Modulo before addition: 10 % 3 + 2" "SUCCESS:3" -e '10 % 3 + 2'
run_test "TC-PARSE-01" "Parser" "Leading decimal dot: .5 + .5" "SUCCESS:1" -e '.5 + .5'
run_test "TC-PARSE-02" "Parser" "Trailing decimal dot: 5. + 5." "SUCCESS:10" -e '5. + 5.'
run_test "TC-PARSE-03" "Parser" "Scientific notation lowercase and uppercase: 1e3 + 1E3" "SUCCESS:2000" -e '1e3 + 1E3'
run_test "TC-PARSE-04" "Parser" "Euler constant coefficient: 1e * 2" "SUCCESS:5.436" -e '1e * 2'
run_test "TC-PARSE-05" "Parser" "Conversion to inches keyword: 5 cm to in" "SUCCESS:1.9685" -e '5 cm to in'
run_test "TC-PARSE-06" "Parser" "Conversion with in in: 5 cm in in" "SUCCESS:1.9685" -e '5 cm in in'
run_test "TC-PARSE-07" "Parser" "Feet and inches notation: 5'10\" to cm" "SUCCESS:177.8 cm" -e '5'"'"'10" to cm'
run_test "TC-PARSE-08" "Parser" "64-bit unsigned hex: 0xffffffffffffffff to hex" "SUCCESS:0xffffffffffffffff" -e '0xffffffffffffffff to hex'
run_test "TC-PARSE-09" "Parser" "Binary and octal literals: 0b1010 + 0o77" "SUCCESS:73" -e '0b1010 + 0o77'
run_test "TC-PARSE-10" "Parser" "Degree symbol conversion: 45° to rad" "SUCCESS:0.785398" -e '45° to rad'
run_test "TC-NUM-01" "Numeric" "Zero to negative power: 0^-1" "ERROR:Division by zero in power" -e '0^-1'
run_test "TC-NUM-02" "Numeric" "Zero to power zero: 0^0" "SUCCESS:1" -e '0^0'
run_test "TC-NUM-03" "Numeric" "Massive exponent limit: 2^10000000" "ERROR:Exponent too large" -e '2^10000000'
run_test "TC-NUM-04" "Numeric" "Exact rational multiplication: 1/3 * 3" "SUCCESS:1" -e '1/3 * 3'
run_test "TC-NUM-05" "Numeric" "Modulo by float zero: 5 % 0.0" "ERROR:Division by zero in modulus" -e '5 % 0.0'
run_test "TC-NUM-06" "Numeric" "Small angle cos precision: 1 - cos(1e-8)" "SUCCESS:0" -e '1 - cos(1e-8)'
run_test "TC-NUM-07" "Numeric" "Exact cube root: cbrt(-8)" "SUCCESS:-2" -e 'cbrt(-8)'
run_test "TC-NUM-08" "Numeric" "Formatting non-finite radix: bin(inf)" "ERROR:Unknown unit" -e 'bin(inf)'
run_test "TC-NUM-09" "Numeric" "Bit shift overflow: 1 << 64" "ERROR:exceeds maximum 63" -e '1 << 64'
run_test "TC-NUM-10" "Numeric" "Negative shift: 1 >> -5" "ERROR:Negative shift amount" -e '1 >> -5'
run_test "TC-COMP-01" "Complex" "Euler's identity: exp(i * pi)" "SUCCESS:-1" -e 'exp(i * pi)'
run_test "TC-COMP-02" "Complex" "Square root of negative one: (-1)^0.5" "SUCCESS:i" -e '(-1)^0.5'
run_test "TC-COMP-03" "Complex" "Square root of negative four: (-4)^0.5" "SUCCESS:2i" -e '(-4)^0.5'
run_test "TC-COMP-04" "Complex" "Imaginary unit power: i^i" "SUCCESS:0.207879" -e 'i^i'
run_test "TC-COMP-05" "Complex" "Reciprocal of imaginary unit: 1 / i" "SUCCESS:-i" -e '1 / i'
run_test "TC-COMP-06" "Complex" "Reciprocal of 1 + i: 1 / (1 + i)" "SUCCESS:0.5 - 0.5i" -e '1 / (1 + i)'
run_test "TC-COMP-07" "Complex" "Natural log of negative one: ln(-1)" "SUCCESS:3.14159265" -e 'ln(-1)'
run_test "TC-COMP-08" "Complex" "Log10 of negative one: log10(-1)" "SUCCESS:1.364376" -e 'log10(-1)'
run_test "TC-COMP-09" "Complex" "Sine of imaginary unit: sin(i)" "SUCCESS:1.1752" -e 'sin(i)'
run_test "TC-COMP-10" "Complex" "Cosine of imaginary unit: cos(i)" "SUCCESS:1.543" -e 'cos(i)'
run_test "TC-UNIT-01" "Units" "Compound velocity conversion: 100 km/h to m/s" "SUCCESS:27.77" -e '100 km/h to m/s'
run_test "TC-UNIT-02" "Units" "Pressure formula: 100 N / 2 m^2 to Pa" "SUCCESS:50 Pa" -e '100 N / 2 m^2 to Pa'
run_test "TC-UNIT-03" "Units" "Data rate conversion: 1 megabit/s * 30 day to gibibyte" "SUCCESS:301.7" -e '1 megabit/s * 30 day to gibibyte'
run_test "TC-UNIT-04" "Units" "Valid dimensional square root: sqrt(4 m^2)" "SUCCESS:2 m" -e 'sqrt(4 m^2)'
run_test "TC-UNIT-05" "Units" "Invalid dimensional square root: sqrt(4 m)" "ERROR:not divisible by 2" -e 'sqrt(4 m)'
run_test "TC-UNIT-06" "Units" "Valid dimensional cube root: cbrt(8 m^3)" "SUCCESS:2 m" -e 'cbrt(8 m^3)'
run_test "TC-UNIT-07" "Units" "Invalid dimensional cube root: cbrt(8 m)" "ERROR:not divisible by 3" -e 'cbrt(8 m)'
run_test "TC-UNIT-08" "Units" "Incompatible dimensions: 5 m + 2 s" "ERROR:Dimensional mismatch" -e '5 m + 2 s'
run_test "TC-UNIT-09" "Units" "Ohm's law reduction: (5 V) / (2 A) to ohm" "SUCCESS:2.5 ohm" -e '(5 V) / (2 A) to ohm'
run_test "TC-UNIT-10" "Units" "Unit preservation in rounding: round(10.5 m)" "SUCCESS:11 m" -e 'round(10.5 m)'
run_test "TC-TEMP-01" "Temperature" "Boiling point Celsius to Fahrenheit: 100 degC to degF" "SUCCESS:212 degF" -e '100 degC to degF'
run_test "TC-TEMP-02" "Temperature" "Freezing point Celsius to Kelvin: 0 degC to K" "SUCCESS:273.15 K" -e '0 degC to K'
run_test "TC-TEMP-03" "Temperature" "Crossing point: -40 degC to degF" "SUCCESS:-40 degF" -e '-40 degC to degF'
run_test "TC-TEMP-04" "Temperature" "Preserve unconverted Celsius: 10 degC" "SUCCESS:10 degC" -e '10 degC'
run_test "TC-TEMP-05" "Temperature" "Preserve unconverted Fahrenheit: 100 degF" "SUCCESS:100 degF" -e '100 degF'
run_test "TC-TEMP-06" "Temperature" "Identity temperature: 10 degC to degC" "SUCCESS:10 degC" -e '10 degC to degC'
run_test "TC-TEMP-07" "Temperature" "Unicode degree Celsius to Fahrenheit: 10 °C to degF" "SUCCESS:50 degF" -e '10 °C to degF'
run_test "TC-TEMP-08" "Temperature" "Absolute zero Kelvin to Celsius: 0 K to degC" "SUCCESS:-273.15 degC" -e '0 K to degC'
run_test "TC-CURR-01" "Currency" "Euro to USD conversion: 1 EUR to USD" "SUCCESS:USD" -e '1 EUR to USD'
run_test "TC-CURR-02" "Currency" "Roundtrip currency: 100 USD to EUR to USD" "SUCCESS:100 USD" -e '100 USD to EUR to USD'
run_test "TC-CURR-03" "Currency" "Decimal currency conversion: 100.5 EUR to USD" "SUCCESS:104.319 USD" -e '100.5 EUR to USD'
run_test "TC-CURR-04" "Currency" "Offline currency baseline: 100 EUR to USD" "SUCCESS:103.8 USD" -e '100 EUR to USD'
run_test "TC-CAS-01" "CAS" "First derivative: diff(x^3, x)" "SUCCESS:3 * x ^ 2" -e 'diff(x^3, x)'
run_test "TC-CAS-02" "CAS" "Second derivative: diff(diff(x^3, x), x)" "SUCCESS:6x" -e 'diff(diff(x^3, x), x)'
run_test "TC-CAS-03" "CAS" "Tangent derivative: diff(tan(x), x)" "SUCCESS:1 / cos(x) ^ 2" -e 'diff(tan(x), x)'
run_test "TC-CAS-04" "CAS" "Arctangent derivative: diff(atan(x), x)" "SUCCESS:1 / 1 + x ^ 2" -e 'diff(atan(x), x)'
run_test "TC-CAS-05" "CAS" "Arcsine derivative: diff(asin(x), x)" "SUCCESS:1 / sqrt(1 - x ^ 2)" -e 'diff(asin(x), x)'
run_test "TC-CAS-06" "CAS" "Sine root solving: sin(x) = 0" "SUCCESS:x ≈ 0.000000" -e 'sin(x) = 0'
run_test "TC-CAS-07" "CAS" "Arctangent root solving: atan(x) = 1" "SUCCESS:x ≈ 1.557408" -e 'atan(x) = 1'
run_test "TC-CAS-08" "CAS" "Cubic root solving: x^3 - 27 = 0" "SUCCESS:x ≈ 3.000000" -e 'x^3 - 27 = 0'
run_test "TC-CAS-09" "CAS" "Factor polynomial: factor(x^2 - 9)" "SUCCESS:(x - 3)(x + 3)" -e 'factor(x^2 - 9)'
run_test "TC-CAS-10" "CAS" "Prime factors: 12 to factors" "SUCCESS:2^2 * 3" -e '12 to factors'
run_test "TC-UNC-01" "Uncertainty" "Zero nominal multiplication: (0 +/- 1) * (5 +/- 1)" "SUCCESS:0.0000±5.0000" -e '(0 +/- 1) * (5 +/- 1)'
run_test "TC-UNC-02" "Uncertainty" "Zero nominal division: (0 +/- 1) / (5 +/- 1)" "SUCCESS:0.0000±0.2000" -e '(0 +/- 1) / (5 +/- 1)'
run_test "TC-UNC-03" "Uncertainty" "Zero error multiplication: (10 +/- 0) * (5 +/- 1)" "SUCCESS:50.0000±10.0000" -e '(10 +/- 0) * (5 +/- 1)'
run_test "TC-UNC-04" "Uncertainty" "Square root uncertainty: sqrt(4 +/- 0.4)" "SUCCESS:2.0000±0.1000" -e 'sqrt(4 +/- 0.4)'
run_test "TC-UNC-05" "Uncertainty" "Power uncertainty: (2 +/- 0.1)^2" "SUCCESS:4.0000±0.4000" -e '(2 +/- 0.1)^2'
run_test "TC-UNC-06" "Uncertainty" "Uncertainty with units: 10 m +/- 1 m" "SUCCESS:10.0000±1.0000 m" -e '10 m +/- 1 m'
run_test "TC-UNC-07" "Uncertainty" "Normal distribution at mean: normdist(0)" "SUCCESS:0.5" -e 'normdist(0)'
run_test "TC-UNC-08" "Uncertainty" "Normal distribution invalid std: normdist(0, 0, 0)" "ERROR:Standard deviation must be positive" -e 'normdist(0, 0, 0)'
run_test "TC-CLI-01" "CLI" "Piped stdin evaluation: 5 + 5" "SUCCESS:10" --pipe '5 + 5'
run_test "TC-CLI-02" "CLI" "Raw mode hex: -r -b hex -e 255" "SUCCESS:ff" -r -b hex -e 255
run_test "TC-CLI-03" "CLI" "Negative expression escape: -- -5 + 10" "SUCCESS:5" -- '-5 + 10'
run_test "TC-CLI-04" "CLI" "Empty expression rejection: -e \"\"" "ERROR:Empty input" -e ''
run_test "TC-CLI-05" "CLI" "Division by zero pipe safety: 1/0" "ERROR:Division by zero" -e 1/0
run_test "TC-AUDIT-01" "Audit" "Affine temperature subtraction: 20 degC - 10 degC" "SUCCESS:10 K" -e '20 degC - 10 degC'
run_test "TC-AUDIT-02" "Audit" "Affine temperature addition rejection: 10 degC + 20 degC" "ERROR:Cannot add two affine temperatures" -e '10 degC + 20 degC'
run_test "TC-AUDIT-03" "Audit" "Bitwise float rejection: 2.5 & 3" "ERROR:Bitwise operations require integer operands" -e '2.5 & 3'
run_test "TC-AUDIT-04" "Audit" "Malformed hexadecimal rejection: 0x" "ERROR:Malformed hexadecimal literal" -e '0x'
run_test "TC-AUDIT-05" "Audit" "Empty diff argument rejection: diff()" "ERROR:Function 'diff' requires at least 1 argument" -e 'diff()'
run_test "TC-AUDIT-06" "Audit" "Identity equation: x = x" "SUCCESS:Identity: all real numbers satisfy this equation" -e 'x = x'
run_test "TC-AUDIT-07" "Audit" "Contradiction equation: x = x + 5" "ERROR:Contradiction: equation has no solution" -e 'x = x + 5'
run_test "TC-AUDIT-08" "Audit" "Dimensional power fractional check: (5 m)^2.5" "ERROR:Non-integer dimensional exponent" -e '(5 m)^2.5'
run_test "TC-M2-01" "M2" "Vector addition: [1, 2, 3] + [4, 5, 6]" "SUCCESS:[5, 7, 9]" -e '[1, 2, 3] + [4, 5, 6]'
run_test "TC-M2-02" "M2" "Scalar multiplication: 2 * [1, 2, 3]" "SUCCESS:[2, 4, 6]" -e '2 * [1, 2, 3]'
run_test "TC-M2-03" "M2" "Dot product: dot([1, 2, 3], [4, 5, 6])" "SUCCESS:32" -e 'dot([1, 2, 3], [4, 5, 6])'
run_test "TC-M2-04" "M2" "Cross product: cross([1, 0, 0], [0, 1, 0])" "SUCCESS:[0, 0, 1]" -e 'cross([1, 0, 0], [0, 1, 0])'
run_test "TC-M2-05" "M2" "Vector norm: norm([3, 4])" "SUCCESS:5" -e 'norm([3, 4])'
run_test "TC-M2-06" "M2" "Matrix multiplication: [[1, 2], [3, 4]] * [[5], [6]]" "SUCCESS:[[17], [39]]" -e '[[1, 2], [3, 4]] * [[5], [6]]'
run_test "TC-M2-07" "M2" "Matrix determinant: det([[1, 2], [3, 4]])" "SUCCESS:-2" -e 'det([[1, 2], [3, 4]])'
run_test "TC-M2-08" "M2" "Matrix inverse: inv([[1, 2], [3, 4]])" "SUCCESS:[[-2, 1], [1.5, -0.5]]" -e 'inv([[1, 2], [3, 4]])'
run_test "TC-M2-09" "M2" "Linear system solver: solve([2x + y = 7, x - y = 2], [x, y])" "SUCCESS:[x = 3, y = 1]" -e 'solve([2x + y = 7, x - y = 2], [x, y])'
run_test "TC-M2-10" "M2" "Integration alias: int(sin(x), 0, pi)" "SUCCESS:2" -e 'int(sin(x), 0, pi)'
run_test "TC-M2-AUDIT-01" "M2-Audit" "Nested empty diff panic guard: diff(diff())" "ERROR:requires at least 1 argument" -e 'diff(diff())'
run_test "TC-M2-AUDIT-02" "M2-Audit" "Nested empty factor panic guard: diff(1 + factor())" "ERROR:requires at least 1 argument" -e 'diff(1 + factor())'
run_test "TC-M2-AUDIT-03" "M2-Audit" "Linearity probe bypass: solve([x*(x-1)+y=2, y=2], [x,y])" "ERROR:non-linear" -e 'solve([x * (x - 1) + y = 2, y = 2], [x, y])'
run_test "TC-M2-AUDIT-04" "M2-Audit" "Complex rejection in vector: [i]" "ERROR:only support real numbers" -e '[i]'
run_test "TC-M2-AUDIT-05" "M2-Audit" "Complex rejection in integration: int(sqrt(-1), 0, 1)" "ERROR:only supports real scalar" -e 'int(sqrt(-1), 0, 1)'
run_test "TC-M2-AUDIT-06" "M2-Audit" "Empty matrix row rejection: [[]]" "ERROR:Empty matrix rows" -e '[[]]'
run_test "TC-M2-AUDIT-07" "M2-Audit" "Empty vector transpose rejection: transpose([])" "ERROR:empty vector" -e 'transpose([])'
run_test "TC-M2-AUDIT-08" "M2-Audit" "Dimension mul overflow guard: (5 m)^100 * (5 m)^50" "ERROR:Dimensional exponent overflow" -e '(5 m)^100 * (5 m)^50'
run_test "TC-M2-AUDIT-09" "M2-Audit" "Unit-aware rounding km: round(10.6 km)" "SUCCESS:11 km" -e 'round(10.6 km)'
run_test "TC-M2-AUDIT-10" "M2-Audit" "Unit-aware rounding degC: round(20.6 degC)" "SUCCESS:21 degC" -e 'round(20.6 degC)'

# ------------------------------------------------------------------------------
# FINAL REPORT GENERATION
# ------------------------------------------------------------------------------


# ==============================================================================
# FINAL EXECUTION SUMMARY
# ==============================================================================
echo "================================================================================"
echo "Master Destructive Test Suite Complete"
echo "Total Tests: $TOTAL_TESTS"
echo "Passed:      $PASSED_TESTS"
echo "Failed:      $FAILED_TESTS"
echo "Crashed:     $CRASHED_TESTS"
PASS_RATE=$(awk 'BEGIN {printf "%.1f", ('"$PASSED_TESTS"' / '"$TOTAL_TESTS"') * 100}')
echo "Pass Rate:   ${PASS_RATE}%"
echo "Log file:    $LOG_FILE"
echo "================================================================================"

if [ "$FAILED_TESTS" -eq 0 ] && [ "$CRASHED_TESTS" -eq 0 ]; then
    exit 0
else
    exit 1
fi
