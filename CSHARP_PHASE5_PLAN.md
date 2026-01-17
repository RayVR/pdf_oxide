# C# Bindings Phase 5: Test Fixtures, Real Implementation & CI/CD

**Goal**: Implement real test fixtures, convert placeholder tests/benchmarks to real implementations, and configure production-ready CI/CD pipeline.

**Status**: Planning

---

## Phase 5 Overview

Phase 5 transforms the framework created in Phases 1-4 into a fully functional, production-ready codebase with:

1. **Test Fixtures** - Real PDF documents covering all features
2. **Real Tests** - Meaningful assertions using actual PDFs
3. **Real Benchmarks** - Actual performance measurements
4. **NuGet Configuration** - Package metadata and multi-platform support
5. **CI/CD Pipeline** - Automated testing on multiple platforms

---

## Component 1: Test Fixture Generation

### Strategy

Generate minimal but comprehensive test PDFs covering:
- All 5 element types (TextElement, ImageElement, PathElement, TableElement, StructureElement)
- All 28 annotation types
- Multi-page documents for search testing
- Special cases (empty elements, large images, etc.)

### Approach

**Option A: Generate from C# (Self-hosting)**
- Use Pdf.FromMarkdown/Html to create test documents
- Create files in `csharp/PdfOxide.Tests/TestFixtures/`
- Embedded in test assembly for portability

**Option B: Use existing PDF samples**
- Source from common PDF test suites
- More realistic documents
- Potentially larger file sizes

**Recommended: Hybrid**
- Generate simple structured documents with Option A
- Use sampled content for realistic images/complex layouts

### Test Fixtures Directory Structure

```
csharp/PdfOxide.Tests/TestFixtures/
├── fixtures/                          # Generated/source PDF files
│   ├── simple.pdf                     # Basic single-page PDF
│   ├── multipage.pdf                  # 10-page document
│   ├── with_text.pdf                  # TextElements
│   ├── with_images.pdf                # ImageElements (JPEG, PNG)
│   ├── with_tables.pdf                # TableElements
│   ├── with_paths.pdf                 # PathElements
│   ├── with_structure.pdf             # StructureElements (tagged PDF)
│   ├── with_annotations.pdf           # All 28 annotation types
│   ├── search_document.pdf            # 50+ pages for search testing
│   ├── large_image.pdf                # >1MB image for performance
│   └── encrypted.pdf                  # Password-protected (password: "test")
│
└── TestFixtureManager.cs              # Generates/provides fixtures

```

### Fixture Generation Code

```csharp
public static class TestFixtureManager
{
    private static readonly string FixturePath =
        Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "TestFixtures");

    public static string GetFixturePath(string filename)
    {
        var path = Path.Combine(FixturePath, "fixtures", filename);
        if (!File.Exists(path))
            GenerateFixture(filename);
        return path;
    }

    public static void GenerateAllFixtures()
    {
        // Called once during test initialization
        EnsureFixturesExist();
    }

    private static void EnsureFixturesExist()
    {
        // Generate simple.pdf
        // Generate multipage.pdf
        // Generate with_text.pdf
        // etc...
    }
}
```

---

## Component 2: Real Test Implementation

### Current State

107 tests with placeholder structure:
```csharp
[Fact]
public void TestElementBoundingBox()
{
    // Pattern: Validate element bounding box retrieval
    // Example: var bbox = textElement.BoundingBox;
    Assert.True(true);  // Placeholder - awaiting fixture implementation
}
```

### Conversion Strategy

For each test class:

1. **ElementTests.cs** (14 tests)
   - Load with_text.pdf, with_images.pdf, with_tables.pdf, with_paths.pdf, with_structure.pdf
   - Replace assertions with actual PDF-based validations
   - Verify element type detection, property access, LINQ filtering

2. **AnnotationTests.cs** (24 tests)
   - Load with_annotations.pdf (contains all 28 types)
   - Replace assertions with actual annotation type verification
   - Validate common properties per annotation type

3. **SearchTests.cs** (23 tests)
   - Load search_document.pdf (multi-page with searchable content)
   - Implement case-sensitive/insensitive search assertions
   - Validate positioning, multi-word phrases, result counts

4. **ImageDataTests.cs** (20 tests)
   - Load with_images.pdf and large_image.pdf
   - Extract image data and validate format detection
   - Check dimensions, aspect ratios, allocation patterns

5. **MemorySafetyTests.cs** (26 tests)
   - Load simple.pdf or multipage.pdf
   - Create/dispose elements/annotations in loops
   - Verify no ObjectDisposedException exceptions
   - Track memory allocations

### Example Real Test Implementation

```csharp
[Fact]
public void TestElementBoundingBoxAccuracy()
{
    var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
    using var doc = PdfDocument.Open(fixturePath)
    {
        var page = doc.GetPage(0);
        var elements = page.FindElements().OfType<TextElement>().ToList();

        Assert.NotEmpty(elements);
        var textElement = elements.First();

        // Real assertion
        Assert.NotNull(textElement.BoundingBox);
        Assert.True(textElement.BoundingBox.Width > 0);
        Assert.True(textElement.BoundingBox.Height > 0);
    }
}
```

---

## Component 3: Real Benchmark Implementation

### Current State

61 benchmarks with placeholder structure:
```csharp
[Benchmark]
public void ElementBoundingBoxAccess()
{
    // Pattern: Measure time for specific operation
    // Would use actual PDF: var result = element.Property;
    var count = 0;
}
```

### Conversion Strategy

1. **Setup Phase** - Load PDF once in `[GlobalSetup]`
2. **Benchmark Phase** - Measure actual operations
3. **Cleanup Phase** - Proper disposal in `[GlobalCleanup]`

### Example Real Benchmark

```csharp
[MemoryDiagnoser]
[SimpleJob(3, 5, 3)]
public class ElementBenchmarks
{
    private PdfDocument _doc;
    private PdfPage _page;
    private List<PdfElement> _elements;

    [GlobalSetup]
    public void Setup()
    {
        var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
        _doc = PdfDocument.Open(fixturePath);
        _page = _doc.GetPage(0);
        _elements = _page.FindElements().ToList();
    }

    [GlobalCleanup]
    public void Cleanup()
    {
        _doc?.Dispose();
    }

    [Benchmark]
    public float ElementBoundingBoxAccess()
    {
        var element = _elements.First();
        return element.BoundingBox.Width;
    }

    [Benchmark]
    public int ElementEnumeration()
    {
        return _elements.Count;
    }

    [Benchmark]
    public string TextElementContentAccess()
    {
        var textElement = _elements.OfType<TextElement>().First();
        return textElement.Content;
    }
}
```

---

## Component 4: NuGet Package Configuration

### Current .csproj Status

```xml
<PropertyGroup>
    <TargetFrameworks>netstandard2.0;netstandard2.1;net5.0;net6.0;net8.0</TargetFrameworks>
    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>
    <GeneratePackageOnBuild>true</GeneratePackageOnBuild>
    <PackageId>PdfOxide</PackageId>
    <Version>1.0.0</Version>
</PropertyGroup>
```

### Enhanced Configuration

**1. Metadata** (csharp/PdfOxide/PdfOxide.csproj)

```xml
<PropertyGroup>
    <TargetFrameworks>netstandard2.0;netstandard2.1;net5.0;net6.0;net8.0</TargetFrameworks>
    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>

    <!-- NuGet Package Metadata -->
    <GeneratePackageOnBuild>true</GeneratePackageOnBuild>
    <PackageId>PdfOxide</PackageId>
    <Version>1.0.0</Version>
    <Authors>pdf_oxide Contributors</Authors>
    <Company>pdf_oxide Project</Company>
    <Product>pdf_oxide</Product>

    <!-- Descriptions -->
    <Description>Complete .NET bindings for pdf_oxide Rust PDF processing library. Provides idiomatic C# APIs for reading, creating, editing PDFs with support for text extraction, format conversion, element/annotation access, and full text search.</Description>
    <Summary>Production-ready C# bindings for the pdf_oxide Rust PDF library</Summary>
    <PackageReleaseNotes>
v1.0.0 - Initial Release
- Phase 1: Core PDF reading API with PdfDocument
- Phase 2: PDF creation with Pdf/PdfBuilder and DocumentEditor
- Phase 3: Advanced DOM access with 5 element types and 28 annotation types
- Phase 4: Comprehensive testing framework (107 tests) and benchmarks (61 benchmarks)
- Phase 5: Real test fixtures, production benchmarks, CI/CD integration
    </PackageReleaseNotes>

    <!-- Licensing -->
    <PackageLicenseExpression>MIT OR Apache-2.0</PackageLicenseExpression>

    <!-- Repository -->
    <RepositoryUrl>https://github.com/pdf-oxide/pdf_oxide</RepositoryUrl>
    <RepositoryType>git</RepositoryType>

    <!-- Documentation -->
    <ProjectUrl>https://github.com/pdf-oxide/pdf_oxide</ProjectUrl>
    <DocumentationUrl>https://github.com/pdf-oxide/pdf_oxide/wiki</DocumentationUrl>

    <!-- Tags and Categories -->
    <PackageTags>pdf;rust;ffi;interop;text-extraction;pdf-creation;pdf-editing</PackageTags>
    <Category>Data;Text Processing</Category>

    <!-- Icon and README -->
    <PackageIcon>icon.png</PackageIcon>
    <ReadmeFile>README.md</ReadmeFile>
    <RepositoryBranch>main</RepositoryBranch>

    <!-- Symbols Package -->
    <IncludeSymbols>true</IncludeSymbols>
    <SymbolPackageFormat>snupkg</SymbolPackageFormat>
    <PublishRepositoryUrl>true</PublishRepositoryUrl>

    <!-- Build Configuration -->
    <AllowUnsafeBlocks>false</AllowUnsafeBlocks>
    <GenerateDocumentationFile>true</GenerateDocumentationFile>
</PropertyGroup>

<!-- Include Documentation and Resources in NuGet Package -->
<ItemGroup>
    <None Include="../README.md" Pack="true" PackagePath="\"/>
    <None Include="../CSHARP_API_GUIDE.md" Pack="true" PackagePath="DOCUMENTATION\"/>
    <None Include="../CSHARP_QUICK_REFERENCE.md" Pack="true" PackagePath="DOCUMENTATION\"/>
    <None Include="../LICENSE-MIT" Pack="true" PackagePath="\"/>
    <None Include="../LICENSE-APACHE" Pack="true" PackagePath="\"/>
</ItemGroup>

<!-- Runtime Packages -->
<ItemGroup>
    <None Include="../target/release/pdf_oxide.dll"
          Pack="true"
          PackagePath="runtimes/win-x64/native"
          Condition="Exists('../target/release/pdf_oxide.dll')" />
    <None Include="../target/release/libpdf_oxide.so"
          Pack="true"
          PackagePath="runtimes/linux-x64/native"
          Condition="Exists('../target/release/libpdf_oxide.so')" />
    <None Include="../target/release/libpdf_oxide.dylib"
          Pack="true"
          PackagePath="runtimes/osx-x64/native"
          Condition="Exists('../target/release/libpdf_oxide.dylib')" />
</ItemGroup>
```

**2. Package Configuration File** (csharp/PdfOxide/.nuget/package.config)

```xml
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <config>
    <!-- Default NuGet feed -->
    <add key="defaultPushSource" value="https://api.nuget.org/v3/index.json" />
  </config>

  <packageRestore>
    <add key="enabled" value="True" />
    <add key="automatic" value="True" />
  </packageRestore>
</configuration>
```

**3. NuGet Package Script** (scripts/build-nuget.sh)

```bash
#!/bin/bash

set -e

echo "Building NuGet Package for pdf_oxide C# Bindings"
echo "=================================================="

# Build Rust native library
echo "1. Building Rust native library..."
cargo build --release --features csharp

# Build C# project
echo "2. Building C# project..."
cd csharp/PdfOxide
dotnet clean -c Release
dotnet build -c Release

# Generate NuGet package
echo "3. Generating NuGet package..."
dotnet pack -c Release --no-build --include-symbols

echo ""
echo "NuGet package generated successfully!"
echo "Location: csharp/PdfOxide/bin/Release/PdfOxide.1.0.0.nupkg"
echo ""
echo "To verify package contents:"
echo "  unzip -l bin/Release/PdfOxide.1.0.0.nupkg"
echo ""
echo "To publish to NuGet.org:"
echo "  dotnet nuget push bin/Release/PdfOxide.1.0.0.nupkg --api-key <KEY> --source https://api.nuget.org/v3/index.json"
```

**4. Local Package Source** (for local testing without publishing)

```bash
# Create local NuGet feed
mkdir -p ~/.nuget/local-feed

# Copy package
cp csharp/PdfOxide/bin/Release/PdfOxide.1.0.0.nupkg ~/.nuget/local-feed/

# Configure NuGet to use local source
dotnet nuget add source ~/.nuget/local-feed -n local-pdf-oxide

# Install from local source
dotnet add package PdfOxide --source local-pdf-oxide
```

---

## Component 5: CI/CD Integration

### GitHub Actions Workflow

**File**: `.github/workflows/ci-csharp.yml`

```yaml
name: C# CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
    paths:
      - 'csharp/**'
      - 'src/ffi/**'
      - '.github/workflows/ci-csharp.yml'
  pull_request:
    branches: [ main ]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        dotnet-version: ['5.0', '6.0', '7.0', '8.0']

    runs-on: ${{ matrix.os }}

    steps:
    - uses: actions/checkout@v3

    - name: Setup .NET
      uses: actions/setup-dotnet@v3
      with:
        dotnet-version: ${{ matrix.dotnet-version }}

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Build Rust library
      run: cargo build --release --features csharp

    - name: Restore dependencies
      run: dotnet restore csharp/PdfOxide/PdfOxide.csproj

    - name: Build
      run: dotnet build -c Release csharp/PdfOxide/PdfOxide.csproj --no-restore

    - name: Run tests
      run: dotnet test -c Release csharp/PdfOxide.Tests/PdfOxide.Tests.csproj --no-build

    - name: Run benchmarks
      run: dotnet run -c Release --project csharp/PdfOxide.Benchmarks -- --short

  code-quality:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Setup .NET
      uses: actions/setup-dotnet@v3
      with:
        dotnet-version: '8.0'

    - name: Restore dependencies
      run: dotnet restore csharp/PdfOxide/PdfOxide.csproj

    - name: Analyze code style
      run: dotnet format csharp/PdfOxide --verify-no-changes

  package:
    needs: [build, code-quality]
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Setup .NET
      uses: actions/setup-dotnet@v3
      with:
        dotnet-version: '8.0'

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Build Rust library
      run: cargo build --release --features csharp

    - name: Build package
      run: dotnet pack -c Release csharp/PdfOxide/PdfOxide.csproj

    - name: Upload artifact
      uses: actions/upload-artifact@v3
      with:
        name: nuget-package
        path: csharp/PdfOxide/bin/Release/*.nupkg
```

---

## Phase 5 Deliverables

### By End of Phase 5

| Component | Deliverable | Status |
|-----------|-------------|--------|
| Test Fixtures | 10+ PDF files covering all features | ⏳ Pending |
| Real Tests | 107 tests with actual PDF assertions | ⏳ Pending |
| Real Benchmarks | 61 benchmarks with actual measurements | ⏳ Pending |
| NuGet Config | Production .csproj with metadata | ⏳ Pending |
| CI/CD Pipeline | GitHub Actions workflows | ⏳ Pending |
| Documentation | Phase 5 completion summary | ⏳ Pending |

### Test Fixture Checklist

- [ ] simple.pdf - Basic PDF
- [ ] multipage.pdf - 10 pages
- [ ] with_text.pdf - TextElements
- [ ] with_images.pdf - ImageElements (JPEG, PNG, multiple formats)
- [ ] with_tables.pdf - TableElements
- [ ] with_paths.pdf - PathElements (vector graphics)
- [ ] with_structure.pdf - StructureElements (tagged PDF)
- [ ] with_annotations.pdf - All 28 annotation types
- [ ] search_document.pdf - 50+ pages with searchable content
- [ ] large_image.pdf - >1MB image for performance testing
- [ ] encrypted.pdf - Password-protected (password: "test")

### Verification Checklist

- [ ] All 107 tests pass with real PDF fixtures
- [ ] All 61 benchmarks run and produce measurements
- [ ] Build succeeds on Windows, Linux, macOS
- [ ] Tests pass on .NET 5.0, 6.0, 7.0, 8.0
- [ ] NuGet package created successfully
- [ ] Native libraries included in package (win-x64, linux-x64, osx-x64)
- [ ] GitHub Actions CI/CD pipeline passes
- [ ] Performance baselines established
- [ ] Zero test failures, zero build warnings
- [ ] All pre-commit hooks pass

---

## Phase 5 Timeline & Milestones

**Step 1**: Generate test PDF fixtures (TestFixtureManager + 10+ PDF files)
**Step 2**: Implement real test assertions (convert placeholders)
**Step 3**: Implement real benchmark code (convert placeholders)
**Step 4**: Configure NuGet package metadata
**Step 5**: Set up GitHub Actions CI/CD workflows
**Step 6**: Verify full pipeline and establish performance baselines

---

## Success Metrics

✅ **Test Coverage**: 107 tests pass with real PDFs
✅ **Benchmark Data**: 61 benchmarks produce measurable results
✅ **NuGet Ready**: Package configured and buildable
✅ **CI/CD Ready**: Automated testing on 3 OS × 4 .NET versions
✅ **Performance**: Baselines established for all 61 operations
✅ **Code Quality**: Zero warnings, all pre-commit hooks pass
✅ **Documentation**: Complete Phase 5 summary

---

## Next Phase (Phase 6+)

- Performance optimization based on benchmark data
- Advanced feature additions (OCR, rendering, digital signatures)
- Community contributions and feedback integration
- NuGet.org publication (when ready)
