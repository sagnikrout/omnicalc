#!/usr/bin/env bash
# ==============================================================================
# OMNICALC DESTRUCTIVE TEST HARNESS V2 - EXTREME EDGE CASES & CRASH TESTING
# Target Binary: /home/sagnik/bin/omnicalc
# ==============================================================================

set -u

BINARY="/home/sagnik/bin/omnicalc"
LOG_FILE="/home/sagnik/code/omnicalc/test_results_v2.log"
CRASH_LOG="/home/sagnik/code/omnicalc/crash_repro.log"

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
NC='\033[0m'

echo "================================================================================"
echo "Starting omnicalc Destructive Test Suite v2 (Extreme Boundary & Crash Hunting)"
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

# Run a test with a 2-second timeout to catch hangs
# Arguments:
#   $1: Test ID
#   $2: Category
#   $3: Description
#   $4: Expected outcome type:
#         "SUCCESS:<expected_substring>" -> Must exit 0 and stdout must contain substring
#         "ERROR:<expected_substring>"   -> Must exit != 0 and stderr must contain substring
#         "CRASH_PROBE"                  -> Must NOT crash (exit 101 or 134 is a CRASH!)
#   $5..$N: Arguments to omnicalc
run_test() {
    local test_id="$1"
    local category="$2"
    local description="$3"
    local expectation="$4"
    shift 4

    local pipe_in=""
    if [ "$1" = "--pipe" ]; then
        pipe_in="$2"
        shift 2
    fi

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    local stdout_file
    local stderr_file
    stdout_file=$(mktemp)
    stderr_file=$(mktemp)

    local exit_code=0
    # Execute with timeout 2s preserving exact argument boundaries
    if [ -n "$pipe_in" ]; then
        echo "$pipe_in" | timeout 2s "$BINARY" "$@" > "$stdout_file" 2> "$stderr_file" || exit_code=$?
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

    # Check for crash (panic exit 101, segfault 139, stack overflow/abort 134, timeout 124)
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

# Run deep piped tests (e.g. for stack recursion limits)
run_piped_test() {
    local test_id="$1"
    local category="$2"
    local description="$3"
    local gen_cmd="$4"
    local expectation="$5"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    local stdout_file
    local stderr_file
    stdout_file=$(mktemp)
    stderr_file=$(mktemp)

    local exit_code=0
    eval "$gen_cmd" | timeout 2s "$BINARY" > "$stdout_file" 2> "$stderr_file" || exit_code=$?

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
        log_entry "$test_id" "$category" "$description" "$gen_cmd | $BINARY" "$exit_code" "$stdout" "$stderr" "$status" "$notes"
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

    log_entry "$test_id" "$category" "$description" "$gen_cmd | $BINARY" "$exit_code" "$stdout" "$stderr" "$status" "$notes"
}

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

# ------------------------------------------------------------------------------
# FINAL REPORT GENERATION
# ------------------------------------------------------------------------------
echo "================================================================================"
echo "Destructive Test Suite v2 Complete"
echo "Total Tests: $TOTAL_TESTS"
echo -e "Passed:      ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:      ${RED}$FAILED_TESTS${NC}"
echo -e "Crashed:     ${RED}$CRASHED_TESTS${NC}"
PASS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED_TESTS / $TOTAL_TESTS) * 100}")
echo "Pass Rate:   $PASS_RATE%"
echo "Full Log:    $LOG_FILE"
echo "Crash Log:   $CRASH_LOG"
echo "================================================================================"

exit $FAILED_TESTS
