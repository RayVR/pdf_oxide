# PDF Oxide Java Bindings

Java Native Interface (JNI) bindings for the high-performance PDF Oxide Rust library.

## Building

### Prerequisites

- Java Development Kit (JDK) 8 or later
- Apache Maven 3.6.0 or later
- Rust 1.70+ with Cargo
- Platform-specific build tools:
  - **Linux**: gcc, make
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft Visual C++ Build Tools 2019 or later

### Build Steps

#### Step 1: Build the Rust native library with Java feature

From the project root directory:

```bash
# Build for current platform
cargo build --release --features java

# For cross-platform builds, use:
# cargo build --release --features java --target x86_64-unknown-linux-gnu
# cargo build --release --features java --target aarch64-unknown-linux-gnu
# cargo build --release --features java --target x86_64-apple-darwin
# cargo build --release --features java --target aarch64-apple-darwin
# cargo build --release --features java --target x86_64-pc-windows-msvc
# cargo build --release --features java --target aarch64-pc-windows-msvc
```

#### Step 2: Copy native libraries to Java resources

Create the appropriate directory structure in `java/src/main/resources/natives/`:

```bash
mkdir -p java/src/main/resources/natives/{linux-x86_64,linux-aarch64,macos-x86_64,macos-aarch64,windows-x86_64,windows-aarch64}

# Copy the built libraries:
# cp target/release/libpdf_oxide_jni.so java/src/main/resources/natives/linux-x86_64/
# cp target/release/libpdf_oxide_jni.dylib java/src/main/resources/natives/macos-x86_64/
# cp target/release/pdf_oxide_jni.dll java/src/main/resources/natives/windows-x86_64/
# ... etc for other platforms
```

#### Step 3: Build the Java JAR

```bash
cd java

# Compile
mvn clean compile

# Run tests
mvn test

# Package
mvn package

# Install locally
mvn install
```

### Development

For local development with automatic native library reloading:

```bash
# Watch for changes and rebuild
mvn -Pdev test

# Or with cargo watch in another terminal:
cargo watch -s "cargo build --release --features java"
```

### Project Structure

```
java/
├── pom.xml                                    # Maven configuration
├── src/
│   ├── main/
│   │   ├── java/
│   │   │   └── com/pdfoxide/
│   │   │       ├── core/                      # Core APIs
│   │   │       ├── document/                  # Document editing
│   │   │       ├── dom/                       # DOM navigation
│   │   │       ├── annotations/               # Annotation types
│   │   │       ├── forms/                     # Form field types
│   │   │       ├── geometry/                  # Geometry types
│   │   │       ├── exceptions/                # Exception hierarchy
│   │   │       ├── util/                      # Utilities
│   │   │       └── internal/                  # Internal classes
│   │   └── resources/
│   │       └── natives/                       # Platform-specific libs
│   │           ├── linux-x86_64/
│   │           ├── linux-aarch64/
│   │           ├── macos-x86_64/
│   │           ├── macos-aarch64/
│   │           ├── windows-x86_64/
│   │           └── windows-aarch64/
│   └── test/
│       ├── java/
│       │   └── com/pdfoxide/
│       └── resources/
```

## Current Implementation Status

**Phase 1: JNI Infrastructure (COMPLETE)**
- ✅ Rust JNI module setup with jni-rs
- ✅ Exception mapping (Rust errors → Java exceptions)
- ✅ Java exception hierarchy (6 exception types)
- ✅ NativeHandle for memory management
- ✅ NativeLibraryLoader with platform detection
- ✅ FeatureDetection for optional features
- ✅ Maven build configuration

**Phase 2-8: API Implementation (PENDING)**
- Core Reading API (PdfDocument)
- Universal API (Pdf)
- Document Editing (DocumentEditor)
- DOM Navigation (PdfPage, PdfElement)
- Annotations (20+ types)
- Form Fields (7 types)
- Advanced Features (Search, Compliance, Signatures, OCR, Rendering)

## Native Method Naming Convention

Java native methods follow the pattern: `native_<api_method_name>`

For example:
- `PdfDocument.open(String path)` → `Java_com_pdfoxide_core_PdfDocument_nativeOpen(JNIEnv, jclass, jstring)`
- `FeatureDetection.hasOcr()` → `Java_com_pdfoxide_util_FeatureDetection_nativeHasOcr(JNIEnv, jclass)`

## Memory Management

The Java bindings use the Cleaner API (Java 9+) for automatic resource cleanup:

```java
// Automatic cleanup - no need to explicitly call close()
try (PdfDocument doc = PdfDocument.open("file.pdf")) {
    String text = doc.extractText(0);
    // Cleaner will call the native cleanup function
}

// Or with explicit cleanup
PdfDocument doc = PdfDocument.open("file.pdf");
doc.close(); // Immediately triggers native cleanup
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
