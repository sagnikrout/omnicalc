#!/usr/bin/env python3
import subprocess
import os
import sys

BINARY = "/home/sagnik/bin/omnicalc"

TEST_CASES = [
    ('TC-PREC-01', 'Precedence', 'Unary minus vs power: -2^2', ['-e', '-2^2'], '-4', False),
    ('TC-PREC-02', 'Precedence', 'Grouped power: (-2)^2', ['-e', '(-2)^2'], '4', False),
    ('TC-PREC-03', 'Precedence', 'Right associative power: 2^3^2', ['-e', '2^3^2'], '512', False),
    ('TC-PREC-04', 'Precedence', 'Implicit multiplication binding: 1/2x', ['-e', '1 / 2x'], '1 / 2x', False),
    ('TC-PREC-05', 'Precedence', 'Explicit multiplication: 1/2*x', ['-e', '1 / 2 * 4'], '2', False),
    ('TC-PREC-06', 'Precedence', 'Unit denominator binding: 100 J / 10 s', ['-e', '100 J / 10 s'], '10 W', False),
    ('TC-PREC-07', 'Precedence', 'Explicit unit multiplication: 100 J / 10 * 1 s', ['-e', '100 J / 10 * 1 s'], '10', False),
    ('TC-PREC-08', 'Precedence', 'Factorial function precedence: fact(5) + 3', ['-e', 'fact(5) + 3'], '123', False),
    ('TC-PREC-09', 'Precedence', 'Bitwise shift precedence: 1 << 2 + 3', ['-e', '1 << 2 + 3'], '32', False),
    ('TC-PREC-10', 'Precedence', 'Modulo before addition: 10 % 3 + 2', ['-e', '10 % 3 + 2'], '3', False),
    ('TC-PARSE-01', 'Parser', 'Leading decimal dot: .5 + .5', ['-e', '.5 + .5'], '1', False),
    ('TC-PARSE-02', 'Parser', 'Trailing decimal dot: 5. + 5.', ['-e', '5. + 5.'], '10', False),
    ('TC-PARSE-03', 'Parser', 'Scientific notation lowercase and uppercase: 1e3 + 1E3', ['-e', '1e3 + 1E3'], '2000', False),
    ('TC-PARSE-04', 'Parser', 'Euler constant coefficient: 1e * 2', ['-e', '1e * 2'], '5.436', False),
    ('TC-PARSE-05', 'Parser', 'Conversion to inches keyword: 5 cm to in', ['-e', '5 cm to in'], '1.9685', False),
    ('TC-PARSE-06', 'Parser', 'Conversion with in in: 5 cm in in', ['-e', '5 cm in in'], '1.9685', False),
    ('TC-PARSE-07', 'Parser', 'Feet and inches notation: 5\'10" to cm', ['-e', '5\'10" to cm'], '177.8 cm', False),
    ('TC-PARSE-08', 'Parser', '64-bit unsigned hex: 0xffffffffffffffff to hex', ['-e', '0xffffffffffffffff to hex'], '0xffffffffffffffff', False),
    ('TC-PARSE-09', 'Parser', 'Binary and octal literals: 0b1010 + 0o77', ['-e', '0b1010 + 0o77'], '73', False),
    ('TC-PARSE-10', 'Parser', 'Degree symbol conversion: 45° to rad', ['-e', '45° to rad'], '0.785398', False),
    ('TC-NUM-01', 'Numeric', 'Zero to negative power: 0^-1', ['-e', '0^-1'], 'Division by zero in power', True),
    ('TC-NUM-02', 'Numeric', 'Zero to power zero: 0^0', ['-e', '0^0'], '1', False),
    ('TC-NUM-03', 'Numeric', 'Massive exponent limit: 2^10000000', ['-e', '2^10000000'], 'Exponent too large', True),
    ('TC-NUM-04', 'Numeric', 'Exact rational multiplication: 1/3 * 3', ['-e', '1/3 * 3'], '1', False),
    ('TC-NUM-05', 'Numeric', 'Modulo by float zero: 5 % 0.0', ['-e', '5 % 0.0'], 'Division by zero in modulus', True),
    ('TC-NUM-06', 'Numeric', 'Small angle cos precision: 1 - cos(1e-8)', ['-e', '1 - cos(1e-8)'], '0', False),
    ('TC-NUM-07', 'Numeric', 'Exact cube root: cbrt(-8)', ['-e', 'cbrt(-8)'], '-2', False),
    ('TC-NUM-08', 'Numeric', 'Formatting non-finite radix: bin(inf)', ['-e', 'bin(inf)'], 'Unknown unit', True),
    ('TC-NUM-09', 'Numeric', 'Bit shift overflow: 1 << 64', ['-e', '1 << 64'], 'exceeds maximum 63', True),
    ('TC-NUM-10', 'Numeric', 'Negative shift: 1 >> -5', ['-e', '1 >> -5'], 'Negative shift amount', True),
    ('TC-COMP-01', 'Complex', "Euler's identity: exp(i * pi)", ['-e', 'exp(i * pi)'], '-1', False),
    ('TC-COMP-02', 'Complex', 'Square root of negative one: (-1)^0.5', ['-e', '(-1)^0.5'], 'i', False),
    ('TC-COMP-03', 'Complex', 'Square root of negative four: (-4)^0.5', ['-e', '(-4)^0.5'], '2i', False),
    ('TC-COMP-04', 'Complex', 'Imaginary unit power: i^i', ['-e', 'i^i'], '0.207879', False),
    ('TC-COMP-05', 'Complex', 'Reciprocal of imaginary unit: 1 / i', ['-e', '1 / i'], '-i', False),
    ('TC-COMP-06', 'Complex', 'Reciprocal of 1 + i: 1 / (1 + i)', ['-e', '1 / (1 + i)'], '0.5 - 0.5i', False),
    ('TC-COMP-07', 'Complex', 'Natural log of negative one: ln(-1)', ['-e', 'ln(-1)'], '3.14159265', False),
    ('TC-COMP-08', 'Complex', 'Log10 of negative one: log10(-1)', ['-e', 'log10(-1)'], '1.364376', False),
    ('TC-COMP-09', 'Complex', 'Sine of imaginary unit: sin(i)', ['-e', 'sin(i)'], '1.1752', False),
    ('TC-COMP-10', 'Complex', 'Cosine of imaginary unit: cos(i)', ['-e', 'cos(i)'], '1.543', False),
    ('TC-UNIT-01', 'Units', 'Compound velocity conversion: 100 km/h to m/s', ['-e', '100 km/h to m/s'], '27.77', False),
    ('TC-UNIT-02', 'Units', 'Pressure formula: 100 N / 2 m^2 to Pa', ['-e', '100 N / 2 m^2 to Pa'], '50 Pa', False),
    ('TC-UNIT-03', 'Units', 'Data rate conversion: 1 megabit/s * 30 day to gibibyte', ['-e', '1 megabit/s * 30 day to gibibyte'], '301.7', False),
    ('TC-UNIT-04', 'Units', 'Valid dimensional square root: sqrt(4 m^2)', ['-e', 'sqrt(4 m^2)'], '2 m', False),
    ('TC-UNIT-05', 'Units', 'Invalid dimensional square root: sqrt(4 m)', ['-e', 'sqrt(4 m)'], 'not divisible by 2', True),
    ('TC-UNIT-06', 'Units', 'Valid dimensional cube root: cbrt(8 m^3)', ['-e', 'cbrt(8 m^3)'], '2 m', False),
    ('TC-UNIT-07', 'Units', 'Invalid dimensional cube root: cbrt(8 m)', ['-e', 'cbrt(8 m)'], 'not divisible by 3', True),
    ('TC-UNIT-08', 'Units', 'Incompatible dimensions: 5 m + 2 s', ['-e', '5 m + 2 s'], 'Dimensional mismatch', True),
    ('TC-UNIT-09', 'Units', "Ohm's law reduction: (5 V) / (2 A) to ohm", ['-e', '(5 V) / (2 A) to ohm'], '2.5 ohm', False),
    ('TC-UNIT-10', 'Units', 'Unit preservation in rounding: round(10.5 m)', ['-e', 'round(10.5 m)'], '11 m', False),
    ('TC-TEMP-01', 'Temperature', 'Boiling point Celsius to Fahrenheit: 100 degC to degF', ['-e', '100 degC to degF'], '212 degF', False),
    ('TC-TEMP-02', 'Temperature', 'Freezing point Celsius to Kelvin: 0 degC to K', ['-e', '0 degC to K'], '273.15 K', False),
    ('TC-TEMP-03', 'Temperature', 'Crossing point: -40 degC to degF', ['-e', '-40 degC to degF'], '-40 degF', False),
    ('TC-TEMP-04', 'Temperature', 'Preserve unconverted Celsius: 10 degC', ['-e', '10 degC'], '10 degC', False),
    ('TC-TEMP-05', 'Temperature', 'Preserve unconverted Fahrenheit: 100 degF', ['-e', '100 degF'], '100 degF', False),
    ('TC-TEMP-06', 'Temperature', 'Identity temperature: 10 degC to degC', ['-e', '10 degC to degC'], '10 degC', False),
    ('TC-TEMP-07', 'Temperature', 'Unicode degree Celsius to Fahrenheit: 10 °C to degF', ['-e', '10 °C to degF'], '50 degF', False),
    ('TC-TEMP-08', 'Temperature', 'Absolute zero Kelvin to Celsius: 0 K to degC', ['-e', '0 K to degC'], '-273.15 degC', False),
    ('TC-CURR-01', 'Currency', 'Euro to USD conversion: 1 EUR to USD', ['-e', '1 EUR to USD'], 'USD', False),
    ('TC-CURR-02', 'Currency', 'Roundtrip currency: 100 USD to EUR to USD', ['-e', '100 USD to EUR to USD'], '100 USD', False),
    ('TC-CURR-03', 'Currency', 'Decimal currency conversion: 100.5 EUR to USD', ['-e', '100.5 EUR to USD'], '104.319 USD', False),
    ('TC-CURR-04', 'Currency', 'Offline currency baseline: 100 EUR to USD', ['-e', '100 EUR to USD'], '103.8 USD', False),
    ('TC-CAS-01', 'CAS', 'First derivative: diff(x^3, x)', ['-e', 'diff(x^3, x)'], '3 * x ^ 2', False),
    ('TC-CAS-02', 'CAS', 'Second derivative: diff(diff(x^3, x), x)', ['-e', 'diff(diff(x^3, x), x)'], '6x', False),
    ('TC-CAS-03', 'CAS', 'Tangent derivative: diff(tan(x), x)', ['-e', 'diff(tan(x), x)'], '1 / cos(x) ^ 2', False),
    ('TC-CAS-04', 'CAS', 'Arctangent derivative: diff(atan(x), x)', ['-e', 'diff(atan(x), x)'], '1 / 1 + x ^ 2', False),
    ('TC-CAS-05', 'CAS', 'Arcsine derivative: diff(asin(x), x)', ['-e', 'diff(asin(x), x)'], '1 / sqrt(1 - x ^ 2)', False),
    ('TC-CAS-06', 'CAS', 'Sine root solving: sin(x) = 0', ['-e', 'sin(x) = 0'], 'x ≈ 0.000000', False),
    ('TC-CAS-07', 'CAS', 'Arctangent root solving: atan(x) = 1', ['-e', 'atan(x) = 1'], 'x ≈ 1.557408', False),
    ('TC-CAS-08', 'CAS', 'Cubic root solving: x^3 - 27 = 0', ['-e', 'x^3 - 27 = 0'], 'x ≈ 3.000000', False),
    ('TC-CAS-09', 'CAS', 'Factor polynomial: factor(x^2 - 9)', ['-e', 'factor(x^2 - 9)'], '(x - 3)(x + 3)', False),
    ('TC-CAS-10', 'CAS', 'Prime factors: 12 to factors', ['-e', '12 to factors'], '2^2 * 3', False),
    ('TC-UNC-01', 'Uncertainty', 'Zero nominal multiplication: (0 +/- 1) * (5 +/- 1)', ['-e', '(0 +/- 1) * (5 +/- 1)'], '0.0000±5.0000', False),
    ('TC-UNC-02', 'Uncertainty', 'Zero nominal division: (0 +/- 1) / (5 +/- 1)', ['-e', '(0 +/- 1) / (5 +/- 1)'], '0.0000±0.2000', False),
    ('TC-UNC-03', 'Uncertainty', 'Zero error multiplication: (10 +/- 0) * (5 +/- 1)', ['-e', '(10 +/- 0) * (5 +/- 1)'], '50.0000±10.0000', False),
    ('TC-UNC-04', 'Uncertainty', 'Square root uncertainty: sqrt(4 +/- 0.4)', ['-e', 'sqrt(4 +/- 0.4)'], '2.0000±0.1000', False),
    ('TC-UNC-05', 'Uncertainty', 'Power uncertainty: (2 +/- 0.1)^2', ['-e', '(2 +/- 0.1)^2'], '4.0000±0.4000', False),
    ('TC-UNC-06', 'Uncertainty', 'Uncertainty with units: 10 m +/- 1 m', ['-e', '10 m +/- 1 m'], '10.0000±1.0000 m', False),
    ('TC-UNC-07', 'Uncertainty', 'Normal distribution at mean: normdist(0)', ['-e', 'normdist(0)'], '0.5', False),
    ('TC-UNC-08', 'Uncertainty', 'Normal distribution invalid std: normdist(0, 0, 0)', ['-e', 'normdist(0, 0, 0)'], 'Standard deviation must be positive', True),
    ('TC-CLI-01', 'CLI', 'Piped stdin evaluation: 5 + 5', ['--pipe', '5 + 5'], '10', False),
    ('TC-CLI-02', 'CLI', 'Raw mode hex: -r -b hex -e 255', ['-r', '-b', 'hex', '-e', '255'], 'ff', False),
    ('TC-CLI-03', 'CLI', 'Negative expression escape: -- -5 + 10', ['--', '-5 + 10'], '5', False),
    ('TC-CLI-04', 'CLI', 'Empty expression rejection: -e ""', ['-e', ''], 'Empty input', True),
    ('TC-CLI-05', 'CLI', 'Division by zero pipe safety: 1/0', ['-e', '1/0'], 'Division by zero', True),
]

def run():
    passed = 0
    failed = 0
    crashed = 0
    failures = []

    print(f"Running {len(TEST_CASES)} Master Catalog Integration Tests against {BINARY}...")
    for tid, cat, desc, args, expected, must_error in TEST_CASES:
        cmd = [BINARY]
        pipe_input = None
        if args and args[0] == "--pipe":
            pipe_input = args[1]
        else:
            cmd.extend(args)

        try:
            p = subprocess.run(cmd, input=pipe_input, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=3)
            exit_code = p.returncode
            stdout = p.stdout.strip()
            stderr = p.stderr.strip()

            if exit_code in (101, 134, 139):
                crashed += 1
                failures.append((tid, desc, f"CRASH with exit {exit_code}", stderr))
                print(f"[CRASH] {tid}: {desc}")
                continue

            if must_error:
                if exit_code != 0 and expected in stderr:
                    passed += 1
                else:
                    failed += 1
                    failures.append((tid, desc, f"Expected error with '{expected}', got exit {exit_code}", f"out: {stdout}, err: {stderr}"))
                    print(f"[FAIL]  {tid}: {desc} -> exit={exit_code}, err={stderr}")
            else:
                if exit_code == 0 and expected in stdout:
                    passed += 1
                else:
                    failed += 1
                    failures.append((tid, desc, f"Expected stdout containing '{expected}', got code {exit_code}", f"out: {stdout}, err: {stderr}"))
                    print(f"[FAIL]  {tid}: {desc} -> exit={exit_code}, out={stdout}, err={stderr}")
        except subprocess.TimeoutExpired:
            crashed += 1
            failures.append((tid, desc, "TIMEOUT (>3s)", ""))
            print(f"[TIMEOUT] {tid}: {desc}")
        except Exception as e:
            failed += 1
            failures.append((tid, desc, str(e), ""))
            print(f"[ERROR] {tid}: {desc} -> {e}")

    print("\n" + "=" * 60)
    rate = (passed / len(TEST_CASES)) * 100
    print(f"Master Catalog Test Results: {passed}/{len(TEST_CASES)} Passed ({rate:.1f}%) | Failed: {failed} | Crashed: {crashed}")
    print("=" * 60)

    if failures:
        print("\nFailure Details:")
        for tid, desc, reason, output in failures:
            print(f"  [{tid}] {desc} -> {reason} ({output})")
        sys.exit(1)
    else:
        print("\nAll integrated test cases passed with 100.0% accuracy!")
        sys.exit(0)

if __name__ == "__main__":
    run()
