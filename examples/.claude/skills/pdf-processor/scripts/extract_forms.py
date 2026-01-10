#!/usr/bin/env python3
"""
PDF Form Data Extraction Script

Extracts form field data from PDF forms.
"""

import sys
import json
from pathlib import Path

def extract_form_data(pdf_path: str) -> dict:
    """Extract form data from PDF."""
    try:
        from pypdf import PdfReader
    except ImportError:
        return {
            'error': 'pypdf not installed. Run: pip install pypdf'
        }

    result = {
        'path': pdf_path,
        'fields': {}
    }

    try:
        reader = PdfReader(pdf_path)

        if reader.get_fields():
            for field_name, field in reader.get_fields().items():
                field_info = {}

                # Get field value
                if '/V' in field:
                    field_info['value'] = field['/V']

                # Get field type
                if '/FT' in field:
                    field_type = field['/FT']
                    if field_type == '/Btn':
                        field_info['type'] = 'button'
                    elif field_type == '/Tx':
                        field_info['type'] = 'text'
                    elif field_type == '/Ch':
                        field_info['type'] = 'choice'
                    elif field_type == '/Sig':
                        field_info['type'] = 'signature'

                # Get flags
                if '/Ff' in field:
                    field_info['flags'] = field['/Ff']

                result['fields'][field_name] = field_info
        else:
            result['message'] = 'No form fields found in PDF'

    except Exception as e:
        result['error'] = str(e)

    return result

def main():
    if len(sys.argv) < 2:
        print("Usage: python extract_forms.py <pdf_path>")
        sys.exit(1)

    pdf_path = sys.argv[1]

    if not Path(pdf_path).exists():
        print(f"Error: File not found: {pdf_path}")
        sys.exit(1)

    result = extract_form_data(pdf_path)

    if 'error' in result:
        print(f"Error: {result['error']}")
        sys.exit(1)

    # Output as JSON
    print(json.dumps(result, indent=2))

    sys.exit(0)

if __name__ == '__main__':
    main()
