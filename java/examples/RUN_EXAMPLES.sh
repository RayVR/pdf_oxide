#!/bin/bash
# Run all PDF Oxide Java examples
# Usage: ./RUN_EXAMPLES.sh

set -e

EXAMPLES_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JAR="$EXAMPLES_DIR/../target/pdf-oxide-1.0.0.jar"

echo "================================================"
echo "PDF Oxide Java Examples"
echo "================================================"
echo ""

# Check if JAR exists
if [ ! -f "$JAR" ]; then
    echo "ERROR: JAR file not found: $JAR"
    echo ""
    echo "Please build first:"
    echo "  cd $(dirname $EXAMPLES_DIR)"
    echo "  ./BUILD_JAR.sh"
    exit 1
fi

echo "Using JAR: $JAR"
echo ""

cd "$EXAMPLES_DIR"

# Compile all examples
echo "Compiling examples..."
javac -cp "$JAR" *.java
echo "✓ Compilation complete"
echo ""

# Function to run example
run_example() {
    local name="$1"
    local class="$2"

    echo "================================================"
    echo "Running: $name"
    echo "================================================"
    java -cp ".:$JAR" "$class"
    echo ""
}

# Run all examples
run_example "1. ReadPdf" "ReadPdf" "sample.pdf"
run_example "2. CreatePdf" "CreatePdf"
run_example "3. SearchPdf" "SearchPdf"
run_example "4. ValidatePdfa" "ValidatePdfa"
run_example "5. EditPdf" "EditPdf"
run_example "6. FormHandling" "FormHandling"

echo "================================================"
echo "✅ All examples completed!"
echo "================================================"
echo ""
echo "Generated files:"
ls -lh *.pdf 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "Form data exports:"
ls -lh *.fdf *.xfdf 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}' || echo "  (none yet)"
