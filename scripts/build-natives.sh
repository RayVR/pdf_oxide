#!/bin/bash
# Build and organize native libraries for Java bindings
# Usage: ./scripts/build-natives.sh [--release] [--clean]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
NATIVE_DIR="$PROJECT_ROOT/java/src/main/resources/natives"
BUILD_TYPE="${1:-release}"
CLEAN="${2:---clean}"

echo "================================================"
echo "PDF Oxide Java Native Libraries Build Script"
echo "================================================"
echo "Project Root: $PROJECT_ROOT"
echo "Build Type: $BUILD_TYPE"
echo "Native Destination: $NATIVE_DIR"
echo ""

# Clean previous builds if requested
if [[ "$CLEAN" == "--clean" ]]; then
    echo "Cleaning previous builds..."
    rm -rf "$PROJECT_ROOT/target"
    rm -rf "$NATIVE_DIR"
fi

# Create native directory structure
mkdir -p "$NATIVE_DIR"

# Detect platform
detect_platform() {
    local os_type=""
    local arch=""

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        os_type="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        os_type="macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        os_type="windows"
    else
        echo "Unsupported OS: $OSTYPE"
        exit 1
    fi

    local machine_arch=$(uname -m)
    if [[ "$machine_arch" == "x86_64" ]] || [[ "$machine_arch" == "amd64" ]]; then
        arch="x86_64"
    elif [[ "$machine_arch" == "aarch64" ]] || [[ "$machine_arch" == "arm64" ]]; then
        arch="aarch64"
    else
        echo "Unsupported architecture: $machine_arch"
        exit 1
    fi

    echo "$os_type-$arch"
}

# Get library filename based on OS
get_lib_name() {
    local os_type="$1"
    case "$os_type" in
        linux-*)
            echo "libpdf_oxide_jni.so"
            ;;
        macos-*)
            echo "libpdf_oxide_jni.dylib"
            ;;
        windows-*)
            echo "pdf_oxide_jni.dll"
            ;;
        *)
            echo "Unknown"
            ;;
    esac
}

# Build for current platform
build_current_platform() {
    local platform=$(detect_platform)
    local os_type="${platform%-*}"
    local lib_name=$(get_lib_name "$os_type")

    echo ""
    echo "================================================"
    echo "Building for: $platform"
    echo "Library name: $lib_name"
    echo "================================================"

    # Build Rust native library
    cd "$PROJECT_ROOT"
    if [[ "$BUILD_TYPE" == "release" ]]; then
        echo "Building release version..."
        cargo build --release --features java
    else
        echo "Building debug version..."
        cargo build --features java
    fi

    # Determine output directory
    local build_dir="$PROJECT_ROOT/target/release"
    if [[ "$BUILD_TYPE" != "release" ]]; then
        build_dir="$PROJECT_ROOT/target/debug"
    fi

    # Copy library to native directory
    # Try the JNI name first, fall back to the standard library name
    local src_lib="$build_dir/$lib_name"
    local src_lib_fallback="$build_dir/libpdf_oxide.so"
    if [[ "$os_type" == "macos" ]]; then
        src_lib_fallback="$build_dir/libpdf_oxide.dylib"
    elif [[ "$os_type" == "windows" ]]; then
        src_lib_fallback="$build_dir/pdf_oxide.dll"
    fi

    local dest_dir="$NATIVE_DIR/$platform"

    if [ ! -f "$src_lib" ]; then
        if [ ! -f "$src_lib_fallback" ]; then
            echo "ERROR: Library not found at $src_lib or $src_lib_fallback"
            echo "Available files in $build_dir:"
            ls -la "$build_dir"/*.{so,dylib,dll} 2>/dev/null || echo "  (none found)"
            exit 1
        fi
        src_lib="$src_lib_fallback"
    fi

    mkdir -p "$dest_dir"
    # Copy and rename if necessary
    cp "$src_lib" "$dest_dir/$lib_name"

    echo "✓ Copied $(basename "$src_lib") to $dest_dir/$lib_name"

    # Verify copy
    if [ -f "$dest_dir/$lib_name" ]; then
        local file_info=$(file "$dest_dir/$lib_name")
        echo "✓ Verified: $file_info"
    else
        echo "ERROR: Failed to copy library"
        exit 1
    fi
}

# Build for all platforms (requires cross-compilation setup)
build_all_platforms() {
    echo ""
    echo "================================================"
    echo "Building for all platforms (requires setup)"
    echo "================================================"

    local targets=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-msvc"
        "aarch64-pc-windows-msvc"
    )

    for target in "${targets[@]}"; do
        echo ""
        echo "Building for: $target"

        local os_type=""
        local arch=""

        if [[ "$target" == "x86_64-unknown-linux-gnu" ]]; then
            os_type="linux"; arch="x86_64"
        elif [[ "$target" == "aarch64-unknown-linux-gnu" ]]; then
            os_type="linux"; arch="aarch64"
        elif [[ "$target" == "x86_64-apple-darwin" ]]; then
            os_type="macos"; arch="x86_64"
        elif [[ "$target" == "aarch64-apple-darwin" ]]; then
            os_type="macos"; arch="aarch64"
        elif [[ "$target" == "x86_64-pc-windows-msvc" ]]; then
            os_type="windows"; arch="x86_64"
        elif [[ "$target" == "aarch64-pc-windows-msvc" ]]; then
            os_type="windows"; arch="aarch64"
        fi

        # Skip if target is not available
        if ! rustup target list | grep -q "$target (installed)"; then
            echo "⚠ Target $target not installed. Skipping..."
            continue
        fi

        cd "$PROJECT_ROOT"
        if [[ "$BUILD_TYPE" == "release" ]]; then
            cargo build --release --features java --target "$target"
        else
            cargo build --features java --target "$target"
        fi

        # Determine library name
        local lib_name=""
        if [[ "$os_type" == "linux" ]]; then
            lib_name="libpdf_oxide_jni.so"
        elif [[ "$os_type" == "macos" ]]; then
            lib_name="libpdf_oxide_jni.dylib"
        elif [[ "$os_type" == "windows" ]]; then
            lib_name="pdf_oxide_jni.dll"
        fi

        # Copy library
        local build_dir="$PROJECT_ROOT/target/$target/release"
        if [[ "$BUILD_TYPE" != "release" ]]; then
            build_dir="$PROJECT_ROOT/target/$target/debug"
        fi

        local src_lib="$build_dir/$lib_name"
        local dest_dir="$NATIVE_DIR/$os_type-$arch"

        if [ -f "$src_lib" ]; then
            mkdir -p "$dest_dir"
            cp "$src_lib" "$dest_dir/"
            echo "✓ Copied $lib_name to $dest_dir"
        else
            echo "⚠ Library not found at $src_lib (may have already been built)"
        fi
    done
}

# Show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  --current           Build only for current platform (default)"
    echo "  --all              Build for all platforms (requires cross-compilation setup)"
    echo "  --release          Build release version (default)"
    echo "  --debug            Build debug version"
    echo "  --clean            Clean previous builds"
    echo ""
    echo "Examples:"
    echo "  $0 --current --release    Build release for current platform"
    echo "  $0 --all --release        Build release for all platforms"
    echo "  $0 --current --clean      Build current platform, cleaning previous builds"
}

# Main
if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
    show_usage
    exit 0
fi

# Parse arguments
BUILD_ALL=false
BUILD_RELEASE=true

for arg in "$@"; do
    case "$arg" in
        --all)
            BUILD_ALL=true
            ;;
        --current)
            BUILD_ALL=false
            ;;
        --release)
            BUILD_RELEASE=true
            ;;
        --debug)
            BUILD_RELEASE=false
            ;;
        --clean)
            CLEAN="--clean"
            ;;
        *)
            echo "Unknown option: $arg"
            show_usage
            exit 1
            ;;
    esac
done

# Execute build
if [[ "$BUILD_ALL" == true ]]; then
    build_all_platforms
else
    build_current_platform
fi

# Show final status
echo ""
echo "================================================"
echo "Build completed successfully!"
echo "================================================"
echo ""
echo "Native libraries location:"
ls -lR "$NATIVE_DIR" | grep -E "^-|^total"
echo ""
echo "Next steps:"
echo "1. Build Java bindings: cd java && mvn clean verify"
echo "2. Run tests: mvn test"
echo "3. Package JAR: mvn package"
