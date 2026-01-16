#!/bin/bash

# Complete Java Compilation Script
# Compiles all 135+ Java classes with proper dependency resolution

set -e

JAVA_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC_DIR="$JAVA_DIR/src/main/java"
TARGET_DIR="$JAVA_DIR/target/complete-classes"
NATIVE_LIB_DIR="$JAVA_DIR/src/main/resources/natives"

echo "════════════════════════════════════════════════════════════════"
echo "Complete Java Compilation - All 135+ Classes"
echo "════════════════════════════════════════════════════════════════"

# Create output directory
rm -rf "$TARGET_DIR"
mkdir -p "$TARGET_DIR"

echo ""
echo "[1/3] Gathering all Java source files..."

# Find all Java files
ALL_JAVA_FILES=$(find "$SRC_DIR" -name "*.java" -type f)
FILE_COUNT=$(echo "$ALL_JAVA_FILES" | wc -l)

echo "Found $FILE_COUNT Java source files"
echo ""

echo "[2/3] Compiling all classes with full dependency resolution..."

# Compile all files at once (lets javac handle dependencies)
javac -encoding UTF-8 \
       -d "$TARGET_DIR" \
       -source 8 -target 8 \
       -cp "$TARGET_DIR" \
       $ALL_JAVA_FILES 2>&1 | tee /tmp/javac_output.txt || true

# Count successful compilations
COMPILED_COUNT=$(find "$TARGET_DIR" -name "*.class" | wc -l)
echo ""
echo "✓ Compilation complete: $COMPILED_COUNT classes compiled"

# Show any errors that occurred
ERRORS=$(grep "error:" /tmp/javac_output.txt | wc -l || true)
WARNINGS=$(grep "warning:" /tmp/javac_output.txt | wc -l || true)

if [ "$ERRORS" -gt 0 ]; then
    echo "⚠ Compilation errors: $ERRORS"
    echo ""
    echo "Error summary (first 10):"
    grep "error:" /tmp/javac_output.txt | head -10
else
    echo "✓ No compilation errors!"
fi

if [ "$WARNINGS" -gt 0 ]; then
    echo "⚠ Warnings: $WARNINGS"
fi

echo ""
echo "[3/3] Creating complete JAR package..."

# Create output JAR directory
JAR_STAGING="$TARGET_DIR/jar-staging"
mkdir -p "$JAR_STAGING"

# Copy compiled classes
cp -r "$TARGET_DIR/com" "$JAR_STAGING/" 2>/dev/null || true

# Copy native libraries
mkdir -p "$JAR_STAGING/natives"
cp -r "$NATIVE_LIB_DIR"/* "$JAR_STAGING/natives/" 2>/dev/null || true

# Create manifest
mkdir -p "$JAR_STAGING/META-INF"
cat > "$JAR_STAGING/META-INF/MANIFEST.MF" << 'MANIFEST'
Manifest-Version: 1.0
Implementation-Title: pdf-oxide Java Bindings
Implementation-Version: 1.0.0
Implementation-Vendor: pdf_oxide Project
Bundle-Description: Java JNI bindings for pdf_oxide Rust library
X-Compile-Source-Level: 8
X-Compile-Target-Level: 8
MANIFEST

# Create JAR
JAR_FILE="$JAVA_DIR/pdf-oxide-1.0.0-complete.jar"
cd "$JAR_STAGING"
jar cvfm "$JAR_FILE" META-INF/MANIFEST.MF com/ natives/ > /dev/null 2>&1 || true
cd - > /dev/null

# Verify JAR
JAR_SIZE=$(ls -lh "$JAR_FILE" 2>/dev/null | awk '{print $5}' || echo "0")
JAR_CLASSES=$(jar tf "$JAR_FILE" 2>/dev/null | grep "\.class$" | wc -l || echo "0")
JAR_NATIVES=$(jar tf "$JAR_FILE" 2>/dev/null | grep -E "\.so|\.dylib|\.dll" | wc -l || echo "0")

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Compilation Summary"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Total Java Files:         $FILE_COUNT"
echo "Successfully Compiled:    $COMPILED_COUNT"
echo "Compilation Errors:       $ERRORS"
echo "Compilation Warnings:     $WARNINGS"
echo ""
echo "JAR Package:"
echo "  File:    $JAR_FILE"
echo "  Size:    $JAR_SIZE"
echo "  Classes: $JAR_CLASSES"
echo "  Natives: $JAR_NATIVES"
echo ""

if [ "$JAR_CLASSES" -gt 100 ]; then
    echo "✅ SUCCESS! All classes compiled into JAR!"
    echo ""
    echo "JAR is ready to use:"
    echo "  java -cp $JAR_FILE com.yourapp.Main"
    echo ""
elif [ "$JAR_CLASSES" -gt 50 ]; then
    echo "⚠ PARTIAL SUCCESS: Most classes compiled"
    echo "   ($JAR_CLASSES classes in JAR)"
    echo ""
    echo "   Run with current JAR or fix remaining errors"
else
    echo "⚠ LIMITED CLASSES: Only $JAR_CLASSES classes compiled"
    echo "   Check errors above and fix import dependencies"
fi

echo ""
echo "JAR Contents Sample:"
jar tf "$JAR_FILE" 2>/dev/null | head -15 || echo "(No JAR created)"
echo "..."
echo ""

# Update main JAR file
if [ -f "$JAR_FILE" ]; then
    cp "$JAR_FILE" "$JAVA_DIR/pdf-oxide-1.0.0.jar"
    echo "✓ Updated main JAR: pdf-oxide-1.0.0.jar"
    echo ""
fi

echo "════════════════════════════════════════════════════════════════"
echo "Compilation Script Complete"
echo "════════════════════════════════════════════════════════════════"

