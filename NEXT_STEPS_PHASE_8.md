# Phase 8: Next Steps After Build Completes

**Build Started**: ~10:00 UTC January 15, 2026
**Expected Completion**: ~10:40-11:00 UTC
**Current Status**: Clean build in progress

## What's Happening Now

The Cargo build is performing a **clean rebuild** of the entire pdf_oxide project with the `--features java` flag enabled. This includes:

1. **Cleaning** (113.2 GB of previous builds removed)
2. **Compiling** all Rust dependencies with optional features
3. **Building** all JNI bindings (Phase 7 code - already fixed)
4. **Linking** native library (libpdf_oxide.so / .dylib / .dll)

## Expected Build Output

Once complete (~30-45 minutes), you should see:
```
    Finished `release` profile [optimized] target(s) in X.XXs
```

With:
- Library location: `/home/yfedoseev/projects/pdf_oxide/target/release/libpdf_oxide.so`
- Size: ~5-6 MB
- Contains: All JNI-exported functions with `#[no_mangle]` attribute

## Immediate Next Steps (After Build Completes)

### Step 1: Verify Native Library (1 minute)
```bash
cd /home/yfedoseev/projects/pdf_oxide

# Check file exists and size
ls -lh target/release/libpdf_oxide.so

# Verify JNI symbols
nm target/release/libpdf_oxide.so | grep Java_com_pdfoxide | head -5

# Check library type
file target/release/libpdf_oxide.so
```

**Expected Output**:
```
ELF 64-bit LSB shared object, x86-64, dynamically linked
libpdf_oxide.so: ~5.1 MB
Multiple Java_com_pdfoxide_* symbols visible
```

### Step 2: Organize Natives for Maven (2 minutes)
```bash
cd /home/yfedoseev/projects/pdf_oxide

# Make build script executable
chmod +x scripts/build-natives.sh

# Copy native library to Maven resources
./scripts/build-natives.sh --current --release

# Verify organization
tree java/src/main/resources/natives/ || \
  find java/src/main/resources/natives/ -type f
```

**Expected Structure**:
```
java/src/main/resources/natives/linux-x86_64/
  ├── libpdf_oxide_jni.so (copied and renamed from libpdf_oxide.so)
  └── [5.1 MB file]
```

### Step 3: Build Maven JAR (5 minutes)
```bash
cd /home/yfedoseev/projects/pdf_oxide/java

# Verify Maven is installed
mvn --version

# Check all dependencies are available
mvn dependency:resolve

# Full build and package
mvn clean verify
mvn package

# Verify JAR created
ls -lh target/pdf-oxide-1.0.0.jar
```

**Expected Output**:
```
BUILD SUCCESS
pdf-oxide-1.0.0.jar created [6-8 MB]
```

### Step 4: Verify JAR Contents (2 minutes)
```bash
cd /home/yfedoseev/projects/pdf_oxide/java

# Count classes
jar tf target/pdf-oxide-1.0.0.jar | grep "\.class" | wc -l
# Expected: 150+

# Verify natives embedded
jar tf target/pdf-oxide-1.0.0.jar | grep "natives"
# Expected: natives/linux-x86_64/libpdf_oxide_jni.so

# List main classes
jar tf target/pdf-oxide-1.0.0.jar | grep "com/pdfoxide/core" | head -10
```

### Step 5: Compile and Run Examples (5-10 minutes)
```bash
cd /home/yfedoseev/projects/pdf_oxide/java/examples

# Run automated example runner
./RUN_EXAMPLES.sh

# OR manually:
javac -cp ../target/pdf-oxide-1.0.0.jar *.java
java -cp .:../target/pdf-oxide-1.0.0.jar ReadPdf sample.pdf
java -cp .:../target/pdf-oxide-1.0.0.jar CreatePdf
```

**Expected Results**:
- Sample PDFs created (output_markdown.pdf, etc.)
- Text extraction working
- Format conversions successful (Markdown, HTML)
- All 6 examples run successfully

### Step 6: Run Test Suite (3-5 minutes)
```bash
cd /home/yfedoseev/projects/pdf_oxide/java

# Run all tests
mvn test

# Run specific integration test
mvn test -Dtest=FullWorkflowTest

# Watch for success
# Expected: "Tests run: X, Failures: 0, Errors: 0"
```

## Troubleshooting Guide

### Issue: "Library not found" or UnsatisfiedLinkError
**Cause**: Native library not in resources or not properly named

**Solution**:
```bash
# 1. Verify script copied the file
ls -la java/src/main/resources/natives/*/

# 2. Re-run build script
./scripts/build-natives.sh --current --release

# 3. Clean and rebuild Maven JAR
cd java && mvn clean package
```

### Issue: "Class not found" errors during examples
**Cause**: JAR not in classpath or not properly built

**Solution**:
```bash
# 1. Verify JAR exists
ls -l java/target/pdf-oxide-1.0.0.jar

# 2. Rebuild if missing
cd java && mvn package

# 3. Verify it was added to examples
echo $CLASSPATH | grep pdf-oxide
```

### Issue: "Build fails" during mvn package
**Cause**: Compilation errors in Java classes (unlikely) or missing dependencies

**Solution**:
```bash
# 1. Check for compilation errors
mvn compile

# 2. Run clean verify first
mvn clean verify

# 3. Check Maven version compatibility
mvn --version  # Should be 3.6+

# 4. Force dependency update
mvn -U clean package
```

## File Checklist After Completion

- [ ] `target/release/libpdf_oxide.so` exists (~5.1 MB)
- [ ] `java/src/main/resources/natives/linux-x86_64/libpdf_oxide_jni.so` exists
- [ ] `java/target/pdf-oxide-1.0.0.jar` exists (6-8 MB)
- [ ] `java/examples/*.class` files exist (compiled examples)
- [ ] Sample PDFs created in examples directory:
  - [ ] `output_markdown.pdf`
  - [ ] `output_html.pdf`
  - [ ] `output_text.pdf`
  - [ ] `output_builder.pdf`
  - [ ] `sample_edit.pdf`
  - [ ] `sample_form.pdf`

## Key Commands Summary

```bash
# After build completes - do these 3 things:

# 1. Organize natives
cd /home/yfedoseev/projects/pdf_oxide && ./scripts/build-natives.sh --current --release

# 2. Build JAR
cd java && mvn package

# 3. Test examples
cd examples && ./RUN_EXAMPLES.sh
```

## Progress Tracking

Use this to track where you are in Phase 8:

```
PHASE 8 PROGRESS:
[████████░] 80% Complete

✅ 8.1 - Build automation and CI/CD setup
✅ 8.2 - Integration testing framework
✅ 8.3 - Documentation and guides
✅ 8.3b - Example programs created
⏳ 8.4 - Native library build (IN PROGRESS - ~15 mins left)
⏳ 8.4b - Copy natives (AFTER BUILD)
⏳ 8.5 - Maven JAR build (AFTER NATIVES)
⏳ 8.6 - Run examples (AFTER JAR)
⏳ 8.7 - Test suite (AFTER EXAMPLES)
⏳ 8.8 - Performance testing (FINAL)
```

## Estimated Time to Completion

From build completion:
- Native verification: 1 minute
- Copy natives to Maven: 2 minutes
- Maven build JAR: 5 minutes
- Verify JAR: 2 minutes
- Run examples: 10 minutes
- Run tests: 5 minutes
- Performance testing: 10 minutes

**Total**: ~35-40 minutes after build completes

## When Everything is Complete

After all steps succeed, you will have:

1. ✅ **Production-Ready JAR**: `java/target/pdf-oxide-1.0.0.jar`
   - All 150+ Java classes compiled
   - Platform-specific native library embedded
   - Ready for Maven Central deployment

2. ✅ **Working Examples**: 6 runnable Java programs
   - Demonstrate all major features
   - Can be used as templates for user code
   - Automatically downloaded and run

3. ✅ **Complete Test Suite**: Integration and unit tests
   - Verify all functionality works
   - Memory management validated
   - Performance baselines established

4. ✅ **Documentation**: Comprehensive guides
   - Getting started (developer experience)
   - API reference
   - Code examples for every feature
   - Troubleshooting guide

5. ✅ **CI/CD Pipeline**: Fully automated
   - Cross-platform native builds
   - Maven packaging
   - Test execution
   - Deployment ready

---

**Remember**: The build is already running. Just wait for completion, then execute the 3-command sequence above to finish Phase 8!

Last updated: January 15, 2026, ~10:20 UTC
