---
name: pdf-processor
description: Extract text, fill forms, and merge PDF files. Use when working with PDF documents, forms, or when users mention PDF processing.
version: "2.0.0"
author: "Doc Team <docs@example.com>"
tags:
  - pdf
  - documents
  - forms
  - data-extraction
dependencies: []

allowed_tools:
  - Read
  - "Bash(python:*)"
  - Grep

model: claude-sonnet-4-20250514
---

# PDF Processing

Expert PDF document processing specialist. Extract text, fill forms, merge documents, and manipulate PDFs with precision.

## Quick Start

Extract text from PDF:

```python
import pdfplumber

with pdfplumber.open("document.pdf") as pdf:
    page = pdf.pages[0]
    text = page.extract_text()
    print(text)
```

## Capabilities

### Text Extraction
- Extract plain text from PDF pages
- Preserve layout and formatting
- Handle multi-page documents
- Extract tables from PDFs

### Form Operations
- Fill PDF forms programmatically
- Extract form field data
- Validate form fields
- Flatten filled forms

### Document Manipulation
- Merge multiple PDFs
- split PDFs into pages
- rotate pages
- add watermarks
- compress PDFs

### OCR Integration
- Process scanned PDFs
- Extract text from images
- Improve OCR accuracy
- Handle multiple languages

## Additional Resources

### Form Field Mappings
For detailed form field mappings and instructions, see [forms.md](forms.md).

### API Reference
For complete API documentation, see [reference.md](reference.md).

### Usage Examples
See [examples.md](examples.md) for more usage examples.

## Utility Scripts

Validate PDF files:
```bash
python scripts/validate.py document.pdf
```

Extract form data:
```bash
python scripts/extract_forms.py document.pdf
```

Merge PDFs:
```bash
python scripts/merge.py output.pdf input1.pdf input2.pdf
```

## Requirements

Ensure required packages are installed:
```bash
pip install pypdf pdfplumber pillow reportlab
```

## Troubleshooting

### Common Issues

**Problem**: Script not found
**Solution**: Ensure scripts have execute permissions: `chmod +x scripts/*.py`

**Problem**: Package not installed
**Solution**: Run pip install with required packages

**Problem**: PDF is encrypted
**Solution**: Unlock the PDF first or provide the password

**Problem**: OCR not working
**Solution**: Install tesseract OCR: `apt-get install tesseract-ocr`

## Best Practices

### DO (Recommended)

1. **Validation**
   - Always validate PDF files before processing
   - Check for encryption and permissions
   - Verify file integrity

2. **Error Handling**
   - Handle corrupted PDFs gracefully
   - Provide meaningful error messages
   - Log processing steps

3. **Performance**
   - Process pages in batches for large PDFs
   - Use multiprocessing when possible
   - Cache extracted data

### DON'T (Avoid)

1. **Security Issues**
   - ❌ Process PDFs from untrusted sources without validation
   - ❌ Execute embedded scripts in PDFs
   - ❌ Ignore encryption warnings

2. **Performance Issues**
   - ❌ Load entire PDF into memory unnecessarily
   - ❌ Process pages sequentially when parallel is possible
   - ❌ Ignore memory limits

3. **Quality Issues**
   - ❌ Skip OCR for scanned documents
   - ❌ Ignore layout and formatting
   - ❌ Assume all PDFs have the same structure

---

**Version**: 2.0.0
**Last Updated**: 2025-01-10
**Maintainer**: Doc Team
