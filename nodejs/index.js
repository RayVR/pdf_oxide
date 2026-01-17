// PDF Oxide Node.js bindings - Native module loader

const { platform, arch } = process;

const PLATFORMS = {
  'darwin': {
    'x64': '@pdf-oxide/darwin-x64',
    'arm64': '@pdf-oxide/darwin-arm64',
  },
  'linux': {
    'x64': '@pdf-oxide/linux-x64-gnu',
    'arm64': '@pdf-oxide/linux-arm64-gnu',
  },
  'win32': {
    'x64': '@pdf-oxide/win32-x64-msvc',
    'arm64': '@pdf-oxide/win32-arm64-msvc',
  },
};

function getNativePackageName() {
  const osPackages = PLATFORMS[platform];
  if (!osPackages) {
    throw new Error(`Unsupported platform: ${platform}. Supported platforms: ${Object.keys(PLATFORMS).join(', ')}`);
  }

  const pkg = osPackages[arch];
  if (!pkg) {
    throw new Error(`Unsupported architecture: ${arch} for ${platform}. Supported architectures: ${Object.keys(osPackages).join(', ')}`);
  }

  return pkg;
}

let nativeModule;

function loadNativeModule() {
  if (nativeModule) {
    return nativeModule;
  }

  try {
    // Try loading from platform-specific package first
    const packageName = getNativePackageName();
    try {
      nativeModule = require(packageName);
    } catch (e) {
      // Fallback to local binary if in development
      if (process.env.NODE_ENV === 'development' || process.env.NAPI_DEV) {
        nativeModule = require('./pdf-oxide');
      } else {
        throw e;
      }
    }
    return nativeModule;
  } catch (error) {
    throw new Error(`Failed to load native module: ${error.message}`);
  }
}

// Load and export classes
const native = loadNativeModule();

module.exports = {
  // Version info
  getVersion: native.getVersion,
  getPdfOxideVersion: native.getPdfOxideVersion,

  // Main classes (to be fully implemented)
  PdfDocument: native.PdfDocument,
  Pdf: native.Pdf,
  PdfBuilder: native.PdfBuilder,
  PdfPage: native.PdfPage,

  // Element types (to be fully implemented)
  PdfElement: native.PdfElement,
  PdfText: native.PdfText,
  PdfImage: native.PdfImage,
  PdfPath: native.PdfPath,
  PdfTable: native.PdfTable,
  PdfStructure: native.PdfStructure,

  // Annotation types (to be fully implemented)
  Annotation: native.Annotation,
  TextAnnotation: native.TextAnnotation,
  HighlightAnnotation: native.HighlightAnnotation,
  LinkAnnotation: native.LinkAnnotation,

  // Error types
  PdfError: native.PdfError,
  PdfIoError: native.PdfIoError,
  PdfParseError: native.PdfParseError,
  PdfEncryptionError: native.PdfEncryptionError,
  PdfUnsupportedError: native.PdfUnsupportedError,
  PdfInvalidStateError: native.PdfInvalidStateError,
  PdfDecodeError: native.PdfDecodeError,
  PdfEncodeError: native.PdfEncodeError,
  PdfFontError: native.PdfFontError,
  PdfImageError: native.PdfImageError,
  PdfCircularReferenceError: native.PdfCircularReferenceError,
  PdfRecursionLimitError: native.PdfRecursionLimitError,

  // Types
  PageSize: native.PageSize,
  Rect: native.Rect,
  Point: native.Point,
  Color: native.Color,
  ConversionOptions: native.ConversionOptions,
  SearchOptions: native.SearchOptions,
  SearchResult: native.SearchResult,

  // Utilities
  TextSearcher: native.TextSearcher,
};
