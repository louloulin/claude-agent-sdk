#!/usr/bin/env python3
"""
PDF Merge Script

Merges multiple PDF files into a single PDF.
"""

import sys
from pathlib import Path

def merge_pdfs(output_path: str, *input_paths: str) -> bool:
    """Merge multiple PDFs into one."""
    try:
        from pypdf import PdfMerger
    except ImportError:
        print("Error: pypdf not installed. Run: pip install pypdf")
        return False

    if not input_paths:
        print("Error: No input PDFs provided")
        return False

    # Validate input files
    for pdf_path in input_paths:
        if not Path(pdf_path).exists():
            print(f"Error: File not found: {pdf_path}")
            return False

    try:
        merger = PdfMerger()

        for pdf_path in input_paths:
            print(f"Adding: {pdf_path}")
            merger.append(pdf_path)

        print(f"Writing: {output_path}")
        merger.write(output_path)
        merger.close()

        return True

    except Exception as e:
        print(f"Error: Failed to merge PDFs: {e}")
        return False

def main():
    if len(sys.argv) < 3:
        print("Usage: python merge.py <output_pdf> <input_pdf1> [input_pdf2] ...")
        sys.exit(1)

    output_path = sys.argv[1]
    input_paths = sys.argv[2:]

    if merge_pdfs(output_path, *input_paths):
        print(f"✓ Successfully merged {len(input_paths)} PDFs into {output_path}")
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == '__main__':
    main()
