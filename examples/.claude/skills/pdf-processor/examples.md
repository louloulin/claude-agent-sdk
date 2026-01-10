# PDF Processor - Usage Examples

## Table of Contents

- [Basic Text Extraction](#basic-text-extraction)
- [Table Extraction](#table-extraction)
- [Form Filling](#form-filling)
- [PDF Merging](#pdf-merging)
- [PDF Splitting](#pdf-splitting)
- [OCR Processing](#ocr-processing)
- [Batch Processing](#batch-processing)
- [Real-World Scenarios](#real-world-scenarios)

---

## Basic Text Extraction

### Extract All Text from PDF

```python
import pdfplumber

def extract_text_from_pdf(pdf_path: str) -> str:
    """Extract all text from a PDF file."""
    with pdfplumber.open(pdf_path) as pdf:
        full_text = []
        for page in pdf.pages:
            text = page.extract_text()
            if text:
                full_text.append(text)
        return "\n\n".join(full_text)

# Usage
text = extract_text_from_pdf("document.pdf")
print(text)
```

### Extract Text from Specific Page

```python
import pdfplumber

with pdfplumber.open("document.pdf") as pdf:
    # Get first page (index 0)
    page = pdf.pages[0]
    text = page.extract_text()
    print(text)
```

### Extract Text with Page Numbers

```python
import pdfplumber

def extract_text_with_page_numbers(pdf_path: str) -> dict:
    """Extract text with page numbers."""
    with pdfplumber.open(pdf_path) as pdf:
        result = {}
        for i, page in enumerate(pdf.pages, 1):
            text = page.extract_text()
            if text:
                result[f"Page {i}"] = text
        return result

# Usage
pages = extract_text_with_page_numbers("document.pdf")
for page_num, text in pages.items():
    print(f"{page_num}:\n{text}\n")
```

---

## Table Extraction

### Extract All Tables

```python
import pdfplumber

def extract_all_tables(pdf_path: str) -> list:
    """Extract all tables from PDF."""
    all_tables = []

    with pdfplumber.open(pdf_path) as pdf:
        for page in pdf.pages:
            tables = page.extract_tables()
            if tables:
                all_tables.extend(tables)

    return all_tables

# Usage
tables = extract_all_tables("document.pdf")
for i, table in enumerate(tables):
    print(f"Table {i+1}:")
    for row in table:
        print(row)
    print()
```

### Extract Tables as CSV

```python
import pdfplumber
import csv

def tables_to_csv(pdf_path: str, output_csv: str):
    """Extract tables and save to CSV."""
    with pdfplumber.open(pdf_path) as pdf, \
         open(output_csv, 'w', newline='') as f:

        writer = csv.writer(f)

        for page in pdf.pages:
            tables = page.extract_tables()
            for table in tables:
                for row in table:
                    # Filter out None values
                    clean_row = [cell if cell else '' for cell in row]
                    writer.writerow(clean_row)

# Usage
tables_to_csv("document.pdf", "tables.csv")
```

### Extract Tables with Settings

```python
import pdfplumber

with pdfplumber.open("document.pdf") as pdf:
    page = pdf.pages[0]

    # Custom table extraction settings
    tables = page.extract_tables({
        "vertical_strategy": "text",
        "horizontal_strategy": "text",
        "snap_tolerance": 5,
        "join_tolerance": 5,
    })

    for table in tables:
        for row in table:
            print(row)
```

---

## Form Filling

### Fill PDF Form

```python
from pypdf import PdfReader, PdfWriter

def fill_pdf_form(template_path: str, data: dict, output_path: str):
    """Fill a PDF form with data."""
    reader = PdfReader(template_path)
    writer = PdfWriter()

    # Copy pages from reader to writer
    for page in reader.pages:
        writer.add_page(page)

    # Update form fields
    if reader.get_fields():
        for field in reader.get_fields().values():
            field_name = field.get('/T')
            if field_name and field_name in data:
                writer.update_page_form_field_values(
                    writer.pages[0],
                    {field_name: data[field_name]}
                )

    # Save filled form
    with open(output_path, 'wb') as f:
        writer.write(f)

# Usage
form_data = {
    'name': 'John Doe',
    'email': 'john@example.com',
    'date': '2025-01-10'
}

fill_pdf_form("form_template.pdf", form_data, "filled_form.pdf")
```

### Extract Form Data

```python
from pypdf import PdfReader

def extract_form_data(pdf_path: str) -> dict:
    """Extract data from filled PDF form."""
    reader = PdfReader(pdf_path)

    form_data = {}
    if reader.get_fields():
        for field in reader.get_fields().values():
            field_name = field.get('/T')
            field_value = field.get('/V')
            if field_name and field_value:
                form_data[field_name] = field_value

    return form_data

# Usage
data = extract_form_data("filled_form.pdf")
print(data)
```

---

## PDF Merging

### Merge Multiple PDFs

```python
from pypdf import PdfMerger

def merge_pdfs(pdf_paths: list, output_path: str):
    """Merge multiple PDFs into one."""
    merger = PdfMerger()

    for pdf_path in pdf_paths:
        merger.append(pdf_path)

    merger.write(output_path)
    merger.close()

# Usage
merge_pdfs(
    ["file1.pdf", "file2.pdf", "file3.pdf"],
    "merged.pdf"
)
```

### Merge Specific Pages

```python
from pypdf import PdfMerger

def merge_specific_pages(pdf_paths: list, pages: list, output_path: str):
    """Merge specific pages from PDFs."""
    merger = PdfMerger()

    for pdf_path, page_range in zip(pdf_paths, pages):
        merger.append(pdf_path, pages=page_range)

    merger.write(output_path)
    merger.close()

# Usage: Merge page 1 from file1, pages 2-3 from file2
merge_specific_pages(
    ["file1.pdf", "file2.pdf"],
    [[0], [1, 2]],  # 0-indexed
    "selective_merge.pdf"
)
```

---

## PDF Splitting

### Split PDF into Pages

```python
from pypdf import PdfReader, PdfWriter

def split_pdf_by_pages(pdf_path: str, output_dir: str):
    """Split PDF into individual pages."""
    reader = PdfReader(pdf_path)

    for i, page in enumerate(reader.pages):
        writer = PdfWriter()
        writer.add_page(page)

        output_path = f"{output_dir}/page_{i+1}.pdf"
        with open(output_path, 'wb') as f:
            writer.write(f)

# Usage
split_pdf_by_pages("document.pdf", "output_pages")
```

### Split PDF by Page Ranges

```python
from pypdf import PdfReader, PdfWriter

def split_pdf_by_ranges(pdf_path: str, ranges: list, output_prefix: str):
    """Split PDF into multiple files by page ranges."""
    reader = PdfReader(pdf_path)

    for i, page_range in enumerate(ranges):
        writer = PdfWriter()

        for page_num in page_range:
            if 0 <= page_num < len(reader.pages):
                writer.add_page(reader.pages[page_num])

        output_path = f"{output_prefix}_{i+1}.pdf"
        with open(output_path, 'wb') as f:
            writer.write(f)

# Usage: Split into pages 0-2 and 3-5
split_pdf_by_ranges(
    "document.pdf",
    [[0, 1, 2], [3, 4, 5]],
    "split_part"
)
```

---

## OCR Processing

### OCR Scanned PDF

```python
from pdf2image import convert_from_path
import pytesseract

def ocr_pdf(pdf_path: str, output_txt: str, lang: str = 'eng'):
    """Extract text from scanned PDF using OCR."""
    # Convert PDF to images
    images = convert_from_path(pdf_path)

    # Extract text from each page
    full_text = []
    for i, image in enumerate(images):
        text = pytesseract.image_to_string(image, lang=lang)
        full_text.append(f"--- Page {i+1} ---\n{text}")

    # Save to file
    with open(output_txt, 'w') as f:
        f.write('\n\n'.join(full_text))

# Usage
ocr_pdf("scanned.pdf", "extracted_text.txt")
```

### OCR with Multiple Languages

```python
def ocr_multilingual(pdf_path: str, languages: list):
    """OCR with multiple language support."""
    images = convert_from_path(pdf_path)
    lang_str = '+'.join(languages)

    for i, image in enumerate(images):
        text = pytesseract.image_to_string(image, lang=lang_str)
        print(f"Page {i+1}:\n{text}\n")

# Usage: English and Chinese
ocr_multilingual("document.pdf", ['eng', 'chi_sim'])
```

---

## Batch Processing

### Process Directory of PDFs

```python
import pdfplumber
from pathlib import Path

def batch_extract_text(input_dir: str, output_dir: str):
    """Extract text from all PDFs in a directory."""
    input_path = Path(input_dir)
    output_path = Path(output_dir)
    output_path.mkdir(exist_ok=True)

    for pdf_file in input_path.glob("*.pdf"):
        print(f"Processing: {pdf_file.name}")

        # Extract text
        with pdfplumber.open(pdf_file) as pdf:
            text = []
            for page in pdf.pages:
                page_text = page.extract_text()
                if page_text:
                    text.append(page_text)

        # Save to text file
        output_file = output_path / f"{pdf_file.stem}.txt"
        output_file.write_text('\n\n'.join(text))

# Usage
batch_extract_text("input_pdfs", "extracted_texts")
```

---

## Real-World Scenarios

### Invoice Processing

```python
import pdfplumber
import re
from datetime import datetime

def process_invoice(pdf_path: str) -> dict:
    """Extract invoice information."""
    with pdfplumber.open(pdf_path) as pdf:
        text = pdf.pages[0].extract_text()

    # Extract invoice number
    invoice_num = re.search(r'Invoice\s*[:#]\s*(\w+)', text, re.I)
    invoice_num = invoice_num.group(1) if invoice_num else None

    # Extract date
    date = re.search(r'Date\s*[:]\s*(\d{4}-\d{2}-\d{2})', text, re.I)
    date = date.group(1) if date else None

    # Extract total amount
    total = re.search(r'Total\s*[:]\s*\$?([\d,]+\.?\d*)', text, re.I)
    total = total.group(1) if total else None

    return {
        'invoice_number': invoice_num,
        'date': date,
        'total': total
    }

# Usage
invoice_data = process_invoice("invoice.pdf")
print(invoice_data)
```

### Report Generation

```python
from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas
from reportlab.lib.units import inch

def generate_report(data: dict, output_path: str):
    """Generate a PDF report from data."""
    c = canvas.Canvas(output_path, pagesize=letter)

    # Title
    c.setFont("Helvetica-Bold", 16)
    c.drawString(1*inch, 10*inch, f"Report: {data['title']}")

    # Date
    c.setFont("Helvetica", 12)
    c.drawString(1*inch, 9.5*inch, f"Date: {data['date']}")

    # Content
    y_position = 8*inch
    for line in data['content']:
        c.drawString(1*inch, y_position, line)
        y_position -= 0.3*inch

        # New page if needed
        if y_position < 1*inch:
            c.showPage()
            y_position = 10*inch

    c.save()

# Usage
report_data = {
    'title': 'Monthly Sales',
    'date': '2025-01-10',
    'content': [
        'Total Sales: $50,000',
        'New Customers: 150',
        'Growth: 15%'
    ]
}

generate_report(report_data, "sales_report.pdf")
```

---

**Version**: 2.0.0
**Last Updated**: 2025-01-10
