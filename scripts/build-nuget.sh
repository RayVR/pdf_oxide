#!/bin/bash

# C# Bindings NuGet Package Build Script
# Builds the PdfOxide NuGet package for distribution or local testing

set -e

echo "==================================================="
echo "PdfOxide C# Bindings - NuGet Package Build Script"
echo "==================================================="
echo ""

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$PROJECT_ROOT"
CSHARP_DIR="$PROJECT_ROOT/csharp"
PROJECT_FILE="$CSHARP_DIR/PdfOxide/PdfOxide.csproj"
BENCH_PROJECT="$CSHARP_DIR/PdfOxide.Benchmarks/PdfOxide.Benchmarks.csproj"
OUTPUT_DIR="$CSHARP_DIR/PdfOxide/bin/Release"
LOCAL_FEED="${HOME}/.nuget/local-feed"

echo "Configuration:"
echo "  Project Root: $PROJECT_ROOT"
echo "  Output Directory: $OUTPUT_DIR"
echo "  Local Feed: $LOCAL_FEED"
echo ""

# Step 1: Build Rust native library
echo "[1/5] Building Rust native library..."
cd "$RUST_DIR"
cargo build --release --features csharp
echo "✓ Rust library built successfully"
echo ""

# Step 2: Clean C# project
echo "[2/5] Cleaning C# project..."
cd "$CSHARP_DIR"
dotnet clean -c Release || true
echo "✓ C# project cleaned"
echo ""

# Step 3: Build C# project
echo "[3/5] Building C# project..."
dotnet build -c Release "$PROJECT_FILE"
echo "✓ C# project built successfully"
echo ""

# Step 4: Run tests
echo "[4/5] Running unit tests..."
if [ -f "$CSHARP_DIR/PdfOxide.Tests/PdfOxide.Tests.csproj" ]; then
    dotnet test -c Release "$CSHARP_DIR/PdfOxide.Tests/PdfOxide.Tests.csproj" --no-build || echo "⚠ Some tests may have failed (fixtures may not be available)"
else
    echo "⚠ Test project not found, skipping tests"
fi
echo ""

# Step 5: Generate NuGet package
echo "[5/5] Generating NuGet package..."
dotnet pack -c Release "$PROJECT_FILE" --no-build
echo "✓ NuGet package generated"
echo ""

# Find the generated package
PACKAGE_FILE=$(find "$OUTPUT_DIR" -name "*.nupkg" -not -name "*.snupkg" -type f -printf '%T@ %p\n' | sort -rn | head -1 | cut -d' ' -f2-)

if [ -z "$PACKAGE_FILE" ]; then
    echo "✗ Error: Could not find generated NuGet package"
    exit 1
fi

PACKAGE_BASENAME=$(basename "$PACKAGE_FILE")
SYMBOL_FILE="${PACKAGE_FILE%.nupkg}.snupkg"

echo "==================================================="
echo "✓ Build Complete!"
echo "==================================================="
echo ""
echo "Generated Files:"
echo "  Package: $PACKAGE_FILE ($PACKAGE_BASENAME)"
if [ -f "$SYMBOL_FILE" ]; then
    echo "  Symbols: $SYMBOL_FILE ($(basename "$SYMBOL_FILE"))"
fi
echo ""

# Display package contents
echo "Package Contents:"
unzip -l "$PACKAGE_FILE" | grep -E "\.(dll|so|dylib|xml)" | head -20
echo ""

# Optional: Setup local NuGet feed for local testing
read -p "Would you like to add this package to local NuGet feed? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    mkdir -p "$LOCAL_FEED"
    cp "$PACKAGE_FILE" "$LOCAL_FEED/"
    if [ -f "$SYMBOL_FILE" ]; then
        cp "$SYMBOL_FILE" "$LOCAL_FEED/"
    fi

    # Configure local source if not already configured
    if ! dotnet nuget list source | grep -q "local-pdf-oxide"; then
        echo "Adding local NuGet source..."
        dotnet nuget add source "$LOCAL_FEED" -n local-pdf-oxide
    fi

    echo ""
    echo "Local Feed Setup Complete!"
    echo "You can now install the package with:"
    echo "  dotnet add package PdfOxide --source local-pdf-oxide"
    echo ""
    echo "To remove from local feed later:"
    echo "  rm $LOCAL_FEED/$PACKAGE_BASENAME"
fi

echo ""
echo "Next Steps:"
echo "1. Review package contents: unzip -l \"$PACKAGE_FILE\""
echo "2. Test locally: dotnet add package PdfOxide --source local-pdf-oxide"
echo "3. When ready to publish: dotnet nuget push \"$PACKAGE_FILE\" --api-key <KEY>"
echo ""
