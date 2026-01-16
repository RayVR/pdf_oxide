#!/bin/bash
# Build PDF Oxide Java JAR with embedded native libraries
# Usage: ./BUILD_JAR.sh

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JAVA_DIR="$PROJECT_ROOT/java"
BUILD_DIR="$PROJECT_ROOT/target/release"

echo "================================================"
echo "PDF Oxide Java JAR Build"
echo "================================================"
echo ""

# Step 1: Organize native libraries
echo "Step 1: Organizing native libraries..."
cd "$PROJECT_ROOT"
./scripts/build-natives.sh --current --release

echo ""
echo "Step 2: Building Maven JAR..."
cd "$JAVA_DIR"

# Verify Maven is installed
if ! command -v mvn &> /dev/null; then
    echo "ERROR: Maven is not installed"
    echo "Install with: apt-get install maven (Ubuntu/Debian) or brew install maven (macOS)"
    exit 1
fi

# Compile and package
mvn clean verify

echo ""
echo "Step 3: Building final JAR..."
mvn package

echo ""
echo "================================================"
echo "✅ JAR Build Complete!"
echo "================================================"
echo ""

# Show output
if [ -f "$JAVA_DIR/target/pdf-oxide-1.0.0.jar" ]; then
    echo "JAR Location: $JAVA_DIR/target/pdf-oxide-1.0.0.jar"
    ls -lh "$JAVA_DIR/target/pdf-oxide-1.0.0.jar"

    echo ""
    echo "Contents:"
    jar tf "$JAVA_DIR/target/pdf-oxide-1.0.0.jar" | head -20
    echo "... ($(jar tf $JAVA_DIR/target/pdf-oxide-1.0.0.jar | wc -l) total entries)"

    echo ""
    echo "Next steps:"
    echo "1. Add to classpath: export CLASSPATH=\$CLASSPATH:$JAVA_DIR/target/pdf-oxide-1.0.0.jar"
    echo "2. Run examples: cd $JAVA_DIR/examples"
    echo "3. Compile example: javac -cp ../target/pdf-oxide-1.0.0.jar ReadPdf.java"
    echo "4. Run example: java -cp .:../target/pdf-oxide-1.0.0.jar ReadPdf sample.pdf"
else
    echo "ERROR: JAR file not found at $JAVA_DIR/target/pdf-oxide-1.0.0.jar"
    echo "Build may have failed. Check output above."
    exit 1
fi
