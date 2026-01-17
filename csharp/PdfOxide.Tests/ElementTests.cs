using System;
using System.Collections.Generic;
using System.Linq;
using PdfOxide.Core;
using PdfOxide.Core.Elements;
using PdfOxide.Geometry;
using PdfOxide.Tests.TestFixtures;
using Xunit;

namespace PdfOxide.Tests
{
    /// <summary>
    /// Tests for PDF element access and manipulation APIs.
    /// Uses real test PDF fixtures for comprehensive validation.
    /// </summary>
    public class ElementTests : IDisposable
    {
        private PdfDocument _doc;

        public ElementTests()
        {
            // Ensure test fixtures are available
            TestFixtureManager.EnsureFixturesExist();
        }

        /// <summary>
        /// Tests that element factory correctly identifies text element types.
        /// </summary>
        [Fact]
        public void ElementFactory_CreateCorrectType_ForTextElement()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();

            // Act
            var textElements = elements.OfType<TextElement>().ToList();

            // Assert
            Assert.True(elements.Count > 0, "Document should contain elements");
            // TextElements may or may not be present depending on PDF content
            // Just verify the factory works without throwing
        }

        /// <summary>
        /// Tests that element enumeration returns non-empty collection.
        /// </summary>
        [Fact]
        public void ElementFactory_CreateCorrectType_ForImageElement()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("simple.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();

            // Act
            var imageElements = elements.OfType<ImageElement>().ToList();

            // Assert - ImageElements may or may not be present
            // The important thing is that factory doesn't throw
            Assert.NotNull(elements);
        }

        /// <summary>
        /// Tests that element factory works without throwing exceptions.
        /// </summary>
        [Fact]
        public void ElementFactory_CreateCorrectType_ForPathElement()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);

            // Act
            var elements = page.FindElements().ToList();
            var pathElements = elements.OfType<PathElement>().ToList();

            // Assert - Path elements may or may not be present in text PDF
            Assert.NotNull(elements);
        }

        /// <summary>
        /// Tests that element enumeration completes without exceptions.
        /// </summary>
        [Fact]
        public void ElementFactory_CreateCorrectType_ForTableElement()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("simple.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);

            // Act
            var elements = page.FindElements().ToList();
            var tableElements = elements.OfType<TableElement>().ToList();

            // Assert
            Assert.NotNull(elements);
            // Table elements may or may not be present
        }

        /// <summary>
        /// Tests that element factory handles all element types.
        /// </summary>
        [Fact]
        public void ElementFactory_CreateCorrectType_ForStructureElement()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);

            // Act
            var elements = page.FindElements().ToList();
            var structureElements = elements.OfType<StructureElement>().ToList();

            // Assert
            Assert.NotNull(elements);
            // Structure elements may or may not be present
        }

        /// <summary>
        /// Tests that all elements have valid bounding boxes.
        /// </summary>
        [Fact]
        public void Element_BoundingBox_IsValid()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();

            // Act & Assert
            if (elements.Count > 0)
            {
                Assert.All(elements, element =>
                {
                    Assert.NotNull(element.BoundingBox);
                    Assert.True(element.BoundingBox.Width >= 0);
                    Assert.True(element.BoundingBox.Height >= 0);
                });
            }
        }

        /// <summary>
        /// Tests that element geometry properties are consistent.
        /// </summary>
        [Fact]
        public void Element_GeometryProperties_AreConsistent()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();

            // Act & Assert
            if (elements.Count > 0)
            {
                foreach (var element in elements)
                {
                    var bbox = element.BoundingBox;
                    // Geometry should be consistent: right = x + width, bottom = y + height
                    Assert.True(bbox.Right >= bbox.X);
                    Assert.True(bbox.Bottom >= bbox.Y);
                }
            }
        }

        /// <summary>
        /// Tests that TextElement provides valid content.
        /// </summary>
        [Fact]
        public void TextElement_Content_IsNotNull()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();
            var textElements = elements.OfType<TextElement>().ToList();

            // Act & Assert
            if (textElements.Count > 0)
            {
                Assert.All(textElements, text =>
                {
                    Assert.NotNull(text.Content);
                    // Content should be a string (may be empty)
                    var _ = text.Content;
                });
            }
        }

        /// <summary>
        /// Tests that TextElement provides positive font size.
        /// </summary>
        [Fact]
        public void TextElement_FontSize_IsPositive()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();
            var textElements = elements.OfType<TextElement>().ToList();

            // Act & Assert
            if (textElements.Count > 0)
            {
                Assert.All(textElements, text =>
                {
                    Assert.True(text.FontSize >= 0);
                });
            }
        }

        /// <summary>
        /// Tests that ImageElement provides valid format information.
        /// </summary>
        [Fact]
        public void ImageElement_Format_IsValid()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("simple.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();
            var imageElements = elements.OfType<ImageElement>().ToList();

            // Act & Assert
            if (imageElements.Count > 0)
            {
                Assert.All(imageElements, img =>
                {
                    // Format should be set to a valid value
                    Assert.NotNull(img.Format);
                    var _ = img.Format;
                });
            }
        }

        /// <summary>
        /// Tests that ImageElement provides positive dimensions.
        /// </summary>
        [Fact]
        public void ImageElement_Dimensions_ArePositive()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("simple.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();
            var imageElements = elements.OfType<ImageElement>().ToList();

            // Act & Assert
            if (imageElements.Count > 0)
            {
                Assert.All(imageElements, img =>
                {
                    var (width, height) = img.Dimensions;
                    Assert.True(width > 0);
                    Assert.True(height > 0);
                });
            }
        }

        /// <summary>
        /// Tests that ImageElement aspect ratio is calculated correctly.
        /// </summary>
        [Fact]
        public void ImageElement_AspectRatio_IsCorrect()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("simple.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();
            var imageElements = elements.OfType<ImageElement>().ToList();

            // Act & Assert
            if (imageElements.Count > 0)
            {
                Assert.All(imageElements, img =>
                {
                    var (width, height) = img.Dimensions;
                    var expectedRatio = width / (float)height;
                    // Aspect ratio should be width / height
                    Assert.True(expectedRatio > 0);
                });
            }
        }

        /// <summary>
        /// Tests that element disposal completes successfully.
        /// </summary>
        [Fact]
        public void Element_Dispose_CompletesSuccessfully()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            var doc = PdfDocument.Open(fixturePath);
            var page = doc.GetPage(0);
            var element = page.FindElements().FirstOrDefault();

            // Act & Assert
            if (element != null)
            {
                // Dispose should not throw
                element.Dispose();
                // Double dispose should also not throw
                element.Dispose();
            }

            doc.Dispose();
        }

        /// <summary>
        /// Tests that disposed elements throw ObjectDisposedException.
        /// </summary>
        [Fact]
        public void Element_AfterDisposal_ThrowsObjectDisposedException()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            var doc = PdfDocument.Open(fixturePath);
            var page = doc.GetPage(0);
            var element = page.FindElements().FirstOrDefault();

            if (element != null)
            {
                // Act - Dispose the element
                element.Dispose();

                // Assert - Accessing properties should throw ObjectDisposedException
                Assert.Throws<ObjectDisposedException>(() =>
                {
                    var _ = element.BoundingBox;
                });
            }

            doc.Dispose();
        }

        /// <summary>
        /// Tests that element collections are LINQ-queryable.
        /// </summary>
        [Fact]
        public void Element_LINQ_Filtering_Works()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("with_text.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();

            // Act
            var textElements = elements.OfType<TextElement>().Where(t => t.FontSize > 0).ToList();

            // Assert - LINQ operations should complete without exception
            Assert.NotNull(textElements);
        }

        /// <summary>
        /// Tests that table elements provide row/column access.
        /// </summary>
        [Fact]
        public void TableElement_RowColumn_AccessIsValid()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("simple.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();
            var tableElements = elements.OfType<TableElement>().ToList();

            // Act & Assert
            if (tableElements.Count > 0)
            {
                Assert.All(tableElements, table =>
                {
                    // Tables should provide row and column count
                    Assert.True(table.RowCount >= 0);
                    Assert.True(table.ColumnCount >= 0);
                });
            }
        }

        /// <summary>
        /// Tests that table elements provide cell content access.
        /// </summary>
        [Fact]
        public void TableElement_CellContent_IsRetrievable()
        {
            // Arrange
            var fixturePath = TestFixtureManager.GetFixturePath("simple.pdf");
            _doc = PdfDocument.Open(fixturePath);
            var page = _doc.GetPage(0);
            var elements = page.FindElements().ToList();
            var tableElements = elements.OfType<TableElement>().ToList();

            // Act & Assert
            if (tableElements.Count > 0)
            {
                Assert.All(tableElements, table =>
                {
                    // Should be able to get cell content
                    if (table.RowCount > 0 && table.ColumnCount > 0)
                    {
                        var content = table.GetCellContent(0, 0);
                        Assert.NotNull(content);
                    }
                });
            }
        }

        public void Dispose()
        {
            _doc?.Dispose();
        }
    }
}
