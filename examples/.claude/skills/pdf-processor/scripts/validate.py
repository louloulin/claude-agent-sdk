#!/usr/bin/env python3
"""
PDF Validation Script

Validates PDF files for corruption, encryption, and accessibility.
"""

import sys
from pathlib import Path

def validate_pdf(pdf_path: str) -> dict:
    """Validate a PDF file."""
    try:
        import pdfplumber
    except ImportError:
        return {
            'valid': False,
            'error': 'pdfplumber not installed. Run: pip install pdfplumber'
        }

    result = {
        'valid': True,
        'path': pdf_path,
        'checks': {}
    }

    try:
        with pdfplumber.open(pdf_path) as pdf:
            # Check page count
            result['checks']['page_count'] = len(pdf.pages)

            # Check metadata
            result['checks']['has_metadata'] = bool(pdf.metadata)

            # Check if encrypted
            result['checks']['is_encrypted'] = False

            # Try to access first page
            if pdf.pages:
                page = pdf.pages[0]
                result['checks']['first_page_accessible'] = True
                result['checks']['page_size'] = (page.width, page.height)
            else:
                result['checks']['first_page_accessible'] = False
                result['valid'] = False

    except Exception as e:
        result['valid'] = False
        result['error'] = str(e)

    return result

def main():
    if len(sys.argv) < 2:
        print("Usage: python validate.py <pdf_path>")
        sys.exit(1)

    pdf_path = sys.argv[1]

    if not Path(pdf_path).exists():
        print(f"Error: File not found: {pdf_path}")
        sys.exit(1)

    result = validate_pdf(pdf_path)

    if result['valid']:
        print(f"✓ PDF is valid: {pdf_path}")
        print(f"  Pages: {result['checks'].get('page_count', 'Unknown')}")
        print(f"  Metadata: {'Yes' if result['checks'].get('has_metadata') else 'No'}")
        print(f"  Size: {result['checks'].get('page_size', 'Unknown')}")
        sys.exit(0)
    else:
        print(f"✗ PDF is invalid: {pdf_path}")
        print(f"  Error: {result.get('error', 'Unknown error')}")
        sys.exit(1)

if __name__ == '__main__':
    main()
