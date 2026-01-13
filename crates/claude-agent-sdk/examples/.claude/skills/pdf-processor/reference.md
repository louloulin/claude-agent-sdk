# PDF Processor - Complete API Reference

## Table of Contents

- [pdfplumber](#pdfplumber)
- [PyPDF](#pypdf)
- [reportlab](#reportlab)
- [pillow](#pillow)
- [pytesseract](#pytesseract)

---

## pdfplumber

### Core Classes

#### `pdfplumber.open(path)`

Open a PDF file for processing.

**Parameters:**
- `path` (str): Path to PDF file
- `password` (str, optional): Password for encrypted PDFs

**Returns:** `PDF` object

**Example:**
```python
import pdfplumber

pdf = pdfplumber.open("document.pdf")
pdf.close()
```

#### `PDF` Object

**Properties:**
- `.pages` - List of Page objects
- `.metadata` - PDF metadata dictionary

**Methods:**
```python
# Get page count
len(pdf.pages)

# Access specific page
page = pdf.pages[0]

# Iterate through pages
for page in pdf.pages:
    text = page.extract_text()
```

#### `Page` Object

**Methods:**

##### `extract_text(**kwargs)`

Extract text from page.

**Parameters:**
- `layout` (bool): Preserve layout (default: False)
- `x_tolerance` (int): Horizontal tolerance (default: 2)
- `y_tolerance` (int): Vertical tolerance (default: 2)

**Returns:** str

**Example:**
```python
text = page.extract_text()
text_with_layout = page.extract_text(layout=True)
```

##### `extract_tables(**kwargs)`

Extract tables from page.

**Parameters:**
- `table_settings` (dict): Table extraction settings

**Returns:** List of lists (table rows)

**Example:**
```python
tables = page.extract_tables()
for table in tables:
    for row in table:
        print(row)
```

##### `to_image(**kwargs)`

Convert page to image for visual processing.

**Returns:** `PageImage` object

**Example:**
```python
img = page.to_image()
img.save("page.png")
```

---

## PyPDF

### PdfReader

#### `pypdf.PdfReader(path)`

Read PDF file.

**Example:**
```python
from pypdf import PdfReader

reader = PdfReader("document.pdf")
print(f"Pages: {len(reader.pages)}")
print(f"Metadata: {reader.metadata}")
```

#### Get Page

```python
page = reader.pages[0]
text = page.extract_text()
```

#### Extract Metadata

```python
metadata = reader.metadata
title = metadata.get('/Title', 'Unknown')
author = metadata.get('/Author', 'Unknown')
```

### PdfWriter

#### `pypdf.PdfWriter()`

Create new PDF or merge existing PDFs.

**Example:**
```python
from pypdf import PdfWriter

writer = PdfWriter()

# Add pages from existing PDF
reader = PdfReader("input.pdf")
for page in reader.pages:
    writer.add_page(page)

# Save
with open("output.pdf", "wb") as f:
    writer.write(f)
```

#### Merge PDFs

```python
merger = PdfMerger()

merger.append("file1.pdf")
merger.append("file2.pdf")

merger.write("merged.pdf")
merger.close()
```

---

## reportlab

### Create PDF from Scratch

```python
from reportlab.pdfgen import canvas
from reportlab.lib.pagesizes import letter

c = canvas.Canvas("output.pdf", pagesize=letter)

# Draw text
c.drawString(100, 750, "Hello, World!")

# Draw rectangle
c.rect(100, 700, 200, 100)

# Save
c.save()
```

### Add Images

```python
from reportlab.lib.utils import ImageReader

# Add image
c.drawImage(
    ImageReader("logo.png"),
    100, 600,
    width=200,
    height=100
)
```

---

## pillow

### Process PDF Pages as Images

```python
from PIL import Image
from pdf2image import convert_from_path

# Convert PDF pages to images
images = convert_from_path("document.pdf")

# Process each page
for i, image in enumerate(images):
    # Resize
    image = image.resize((800, 1000))

    # Save
    image.save(f"page_{i}.png")
```

---

## pytesseract

### OCR on Images

```python
from PIL import Image
import pytesseract

# Extract text from image
text = pytesseract.image_to_string(Image.open("page.png"))

# With language
text = pytesseract.image_to_string(
    Image.open("page.png"),
    lang='eng+chi_sim'
)

# Get detailed data
data = pytesseract.image_to_data(Image.open("page.png"))
```

### OCR on PDF

```python
# Convert PDF to images first
images = convert_from_path("scanned.pdf")

# OCR each page
for i, image in enumerate(images):
    text = pytesseract.image_to_string(image)
    print(f"Page {i+1}:\n{text}\n")
```

---

## Advanced Patterns

### Handle Encrypted PDFs

```python
import pdfplumber

try:
    pdf = pdfplumber.open("encrypted.pdf", password="secret123")
except:
    print("Failed to open PDF")
else:
    # Process PDF
    pass
    pdf.close()
```

### Batch Processing

```python
import pdfplumber
from pathlib import Path

pdf_dir = Path("pdfs")

for pdf_path in pdf_dir.glob("*.pdf"):
    with pdfplumber.open(pdf_path) as pdf:
        text = "\n".join(
            page.extract_text()
            for page in pdf.pages
        )

        # Save text
        txt_path = pdf_path.with_suffix('.txt')
        txt_path.write_text(text)
```

### Extract with Layout Preservation

```python
import pdfplumber

with pdfplumber.open("document.pdf") as pdf:
    page = pdf.pages[0]

    # Extract with layout
    text = page.extract_text(
        layout=True,
        x_tolerance=3,
        y_tolerance=3
    )

    # Extract tables
    tables = page.extract_tables({
        "vertical_strategy": "text",
        "horizontal_strategy": "text",
    })
```

---

**Version**: 2.0.0
**Last Updated**: 2025-01-10
