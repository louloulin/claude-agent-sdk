# PDF Form Field Mappings Guide

## Standard PDF Form Fields

### Personal Information
| Field Name | Type | Required | Validation | Example |
|------------|------|----------|------------|---------|
| first_name | text | yes | min 2 chars | John |
| last_name | text | yes | min 2 chars | Doe |
| email | text | yes | valid email | john@example.com |
| phone | text | no | digits only | 1234567890 |
| date_of_birth | text | no | date format | 1990-01-15 |

### Address Fields
| Field Name | Type | Required | Validation | Example |
|------------|------|----------|------------|---------|
| street | text | yes | non-empty | 123 Main St |
| city | text | yes | non-empty | Anytown |
| state | dropdown | yes | valid state code | CA |
| zip | text | yes | 5 digits or 5+4 format | 12345 or 12345-6789 |
| country | dropdown | yes | valid country code | US |

### Agreement Fields
| Field Name | Type | Required | Values | Description |
|------------|------|----------|--------|-------------|
| agree_terms | checkbox | yes | /Yes, /Off | Terms and conditions agreement |
| subscribe | checkbox | no | /Yes, /Off | Newsletter subscription |
| age_confirmation | checkbox | yes | /Yes, /Off | Age 18+ confirmation |
| privacy_policy | checkbox | yes | /Yes, /Off | Privacy policy agreement |

### Employment Fields
| Field Name | Type | Required | Validation | Example |
|------------|------|----------|------------|---------|
| company | text | no | non-empty | Acme Corp |
| job_title | text | no | non-empty | Software Engineer |
| employee_id | text | no | alphanumeric | EMP12345 |
| start_date | text | no | date format | 2024-01-15 |

## Field Access Patterns

### Reading Form Fields

```python
import pypdf

def list_form_fields(pdf_path):
    """List all form fields in a PDF"""
    pdf = pypdf.PdfReader(pdf_path)

    # Check if PDF has forms
    if len(pdf.get_fields()) > 0:
        fields = pdf.get_fields()
        print(f"Found {len(fields)} form fields:\n")

        for field_name, field in fields.items():
            print(f"Field: {field_name}")
            print(f"  Type: {field.field_type}")
            print(f"  Value: {field.value}")
            print(f"  Required: {field.flags.required if hasattr(field, 'flags') else 'Unknown'}")

            # Get dropdown options if applicable
            if hasattr(field, 'export_values') and field.export_values:
                print(f"  Options: {field.export_values}")
            print()
    else:
        print("No form fields found in this PDF")

# Usage
list_form_fields("form.pdf")
```

### Filling Text Fields

```python
import pypdf

def fill_text_field(pdf_path, output_path, field_name, value):
    """Fill a single text field"""
    pdf = pypdf.PdfReader(pdf_path)
    writer = pypdf.PdfWriter()

    # Check if field exists
    if field_name in pdf.forms[0].fields:
        # Fill the field
        pdf.forms[0].fields[field_name].value = str(value)
        print(f"✅ Filled '{field_name}' with: {value}")
    else:
        print(f"❌ Field '{field_name}' not found")
        return

    # Add all pages and save
    writer.add_pages(pdf.pages)
    writer.write(output_path)
    print(f"✅ Saved to: {output_path}")

# Usage
fill_text_field("form.pdf", "filled.pdf", "first_name", "John")
```

### Filling Multi-line Text Fields

```python
def fill_address_field(pdf_path, output_path, street, city, state, zip):
    """Fill address fields with multi-line support"""
    pdf = pypdf.PdfReader(pdf_path)
    writer = pypdf.PdfWriter()

    # Fill individual fields
    pdf.forms[0].fields["street"].value = street
    pdf.forms[0].fields["city"].value = city
    pdf.forms[0].fields["state"].value = state
    pdf.forms[0].fields["zip"].value = zip

    # Alternatively, if there's a single address field:
    # pdf.forms[0].fields["address"].value = f"{street}\n{city}, {state} {zip}"

    writer.add_pages(pdf.pages)
    writer.write(output_path)

# Usage
fill_address_field(
    "form.pdf",
    "filled.pdf",
    "123 Main St\nApt 4B",
    "Anytown",
    "CA",
    "12345"
)
```

### Filling Checkbox Fields

```python
def fill_checkbox(pdf_path, output_path, field_name, checked=True):
    """Fill a checkbox field"""
    pdf = pypdf.PdfReader(pdf_path)
    writer = pypdf.PdfWriter()

    if field_name in pdf.forms[0].fields:
        # Use /Yes for checked, /Off for unchecked
        pdf.forms[0].fields[field_name].value = "/Yes" if checked else "/Off"
        status = "checked" if checked else "unchecked"
        print(f"✅ Checkbox '{field_name}' {status}")
    else:
        print(f"❌ Checkbox field '{field_name}' not found")
        return

    writer.add_pages(pdf.pages)
    writer.write(output_path)

# Usage
fill_checkbox("form.pdf", "filled.pdf", "agree_terms", checked=True)
fill_checkbox("form.pdf", "filled.pdf", "subscribe", checked=False)
```

### Filling Dropdown Fields

```python
def fill_dropdown(pdf_path, output_path, field_name, value):
    """Fill a dropdown field"""
    pdf = pypdf.PdfReader(pdf_path)

    # Get the field
    field = pdf.forms[0].fields.get(field_name)

    if not field:
        print(f"❌ Dropdown field '{field_name}' not found")
        return

    # Check valid options
    if hasattr(field, 'export_values') and field.export_values:
        valid_options = field.export_values
        print(f"Valid options for '{field_name}': {valid_options}")

        if value not in valid_options:
            print(f"❌ Invalid value '{value}'. Must be one of: {valid_options}")
            return
    else:
        print(f"⚠️  No export values found, trying to set value anyway")

    # Set the value
    pdf.forms[0].fields[field_name].value = value
    print(f"✅ Set '{field_name}' to: {value}")

    # Save
    writer = pypdf.PdfWriter()
    writer.add_pages(pdf.pages)
    writer.write(output_path)

# Usage
fill_dropdown("form.pdf", "filled.pdf", "state", "CA")
fill_dropdown("form.pdf", "filled.pdf", "country", "US")
```

## Common Workflows

### Workflow 1: Validate Form Before Filling

```python
import pypdf

def validate_form_fields(pdf_path, required_fields):
    """Validate that all required fields exist"""
    pdf = pypdf.PdfReader(pdf_path)

    if len(pdf.get_fields()) == 0:
        raise ValueError("PDF has no form fields")

    available_fields = set(pdf.get_fields().keys())
    required_set = set(required_fields)

    missing = required_set - available_fields

    if missing:
        raise ValueError(f"Missing required fields: {', '.join(missing)}")

    print(f"✅ All {len(required_fields)} required fields present")
    return True

# Usage
REQUIRED_FIELDS = [
    "first_name",
    "last_name",
    "email",
    "agree_terms"
]

validate_form_fields("application.pdf", REQUIRED_FIELDS)
```

### Workflow 2: Fill Form from Dictionary

```python
def fill_form_from_dict(pdf_path, output_path, data):
    """Fill multiple form fields from a dictionary"""
    pdf = pypdf.PdfReader(pdf_path)
    writer = pypdf.PdfWriter()

    filled = 0
    not_found = []

    for field_name, value in data.items():
        if field_name in pdf.forms[0].fields:
            pdf.forms[0].fields[field_name].value = str(value)
            filled += 1
            print(f"✅ Filled '{field_name}': {value}")
        else:
            not_found.append(field_name)
            print(f"⚠️  Field '{field_name}' not found in PDF")

    # Save
    writer.add_pages(pdf.pages)
    writer.write(output_path)

    print(f"\n📊 Summary:")
    print(f"   Filled: {filled} fields")
    if not_found:
        print(f"   Not found: {', '.join(not_found)}")
    print(f"   Output: {output_path}")

# Usage
form_data = {
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@example.com",
    "phone": "555-1234",
    "agree_terms": "/Yes",
    "subscribe": "/Off",
    "state": "CA",
}

fill_form_from_dict("form.pdf", "filled.pdf", form_data)
```

### Workflow 3: Extract Form Data to Dictionary

```python
def extract_form_data(pdf_path):
    """Extract all form field data to a dictionary"""
    pdf = pypdf.PdfReader(pdf_path)

    if len(pdf.get_fields()) == 0:
        return {}

    data = {}
    for field_name, field in pdf.get_fields().items():
        data[field_name] = {
            "value": field.value,
            "type": field.field_type if hasattr(field, 'field_type') else "unknown",
        }

        # Add extra info if available
        if hasattr(field, 'flags'):
            data[field_name]["required"] = field.flags.required

        if hasattr(field, 'export_values') and field.export_values:
            data[field_name]["options"] = field.export_values

    return data

# Usage
import json
data = extract_form_data("filled_form.pdf")
print(json.dumps(data, indent=2))
```

### Workflow 4: Batch Fill Multiple Forms

```python
import pandas as pd
from pathlib import Path

def batch_fill_forms(template_pdf, data_file, output_dir):
    """Fill multiple forms from a CSV/Excel file"""
    # Read data
    df = pd.read_csv(data_file)  # or pd.read_excel()

    # Create output directory
    output_path = Path(output_dir)
    output_path.mkdir(exist_ok=True)

    results = []

    for index, row in df.iterrows():
        # Generate output filename
        output_file = output_path / f"form_{index + 1}.pdf"

        try:
            # Fill form
            fill_form_from_dict(
                template_pdf,
                str(output_file),
                row.to_dict()
            )
            results.append({"file": output_file.name, "status": "success"})
            print(f"✅ Created: {output_file.name}")
        except Exception as e:
            results.append({"file": output_file.name, "status": f"error: {e}"})
            print(f"❌ Failed: {output_file.name} - {e}")

    # Summary
    success_count = sum(1 for r in results if r["status"] == "success")
    print(f"\n📊 Batch complete: {success_count}/{len(results)} successful")

    return results

# Usage
# Assuming data.csv has columns: first_name, last_name, email, etc.
batch_fill_forms(
    "template.pdf",
    "data.csv",
    "output_forms"
)
```

## Troubleshooting

### Problem: Field Not Found

**Symptom**: KeyError when accessing field
```python
KeyError: 'first_name'
```

**Solution**:
```python
# Check actual field names in PDF
pdf = pypdf.PdfReader("form.pdf")

if len(pdf.get_fields()) > 0:
    print("Available fields:")
    for field_name in pdf.get_fields().keys():
        print(f"  - {field_name}")
```

**Common issues**:
- Field names might have spaces or special characters
- Field names are case-sensitive
- Some PDFs use internal field names different from display names

### Problem: Checkbox Not Working

**Symptom**: Checkbox value doesn't change
```python
pdf.forms[0].fields["agree_terms"].value = "true"  # ❌ Wrong
pdf.forms[0].fields["agree_terms"].value = "/Yes"  # ✅ Correct
```

**Solution**:
Always use `/Yes` for checked and `/Off` for unchecked:

```python
# ✅ Correct checkbox values
CHECKED = "/Yes"
UNCHECKED = "/Off"

pdf.forms[0].fields["agree_terms"].value = CHECKED
pdf.forms[0].fields["subscribe"].value = UNCHECKED
```

**NOT valid**:
- ❌ "true", "false"
- ❌ "yes", "no"
- ❌ "1", "0"
- ❌ True, False (boolean)

### Problem: Dropdown Invalid Value

**Symptom**: Dropdown not selecting value silently fails

**Solution**:
Always check valid options first:

```python
def safe_set_dropdown(pdf, field_name, value):
    """Safely set dropdown value with validation"""
    field = pdf.forms[0].fields.get(field_name)

    if not field:
        print(f"❌ Field '{field_name}' not found")
        return False

    # Get valid options
    if hasattr(field, 'export_values') and field.export_values:
        valid_options = field.export_values

        if value not in valid_options:
            print(f"❌ Invalid value '{value}'")
            print(f"   Valid options: {valid_options}")
            return False

    # Set value
    field.value = value
    print(f"✅ Set '{field_name}' = '{value}'")
    return True

# Usage
safe_set_dropdown(pdf, "state", "CA")  # ✅ Valid
safe_set_dropdown(pdf, "state", "California")  # ❌ Invalid (use code, not name)
```

### Problem: Flattening Forms

**Symptom**: Filled forms can still be edited

**Solution**:
Flatten the form to make it permanent:

```python
from pypdf import PdfReader, PdfWriter
from pypdf.generic import NameObject

def flatten_form_fields(pdf_path, output_path):
    """Flatten form fields to make them permanent"""
    reader = PdfReader(pdf_path)
    writer = PdfWriter()

    for page in reader.pages:
        # Add page
        writer.add_page(page)

    # Flatten all fields
    if "/Annots" in writer.pages[0]:
        for page in writer.pages:
            if "/Annots" in page:
                for annot in page["/Annots"]:
                    if annot.get_object().get("/FT") is not None:  # Form field
                        # Remove the field but keep the appearance
                        annot.get_object()[NameObject("/Ff")] = 1  # ReadOnly

    writer.write(output_path)
    print(f"✅ Flattened form saved to: {output_path}")

# Usage
flatten_form_fields("filled.pdf", "flattened.pdf")
```

**Note**: Flattened forms cannot be edited again. Keep the original filled PDF if you need to make changes.

### Problem: Encrypted PDFs

**Symptom**: Cannot read form fields from password-protected PDF

**Solution**:
```python
def read_encrypted_pdf(pdf_path, password):
    """Read an encrypted PDF"""
    try:
        pdf = pypdf.PdfReader(pdf_path)

        if pdf.is_encrypted:
            if password:
                # Try to decrypt
                if pdf.decrypt(password):
                    print("✅ PDF decrypted successfully")
                else:
                    raise ValueError("Incorrect password")
            else:
                raise ValueError("PDF is encrypted but no password provided")

        return pdf

    except Exception as e:
        print(f"❌ Failed to read PDF: {e}")
        raise

# Usage
pdf = read_encrypted_pdf("protected_form.pdf", "secret_password")
fields = pdf.get_fields()
```

## Best Practices

### DO ✅

1. **Always validate field names**
   ```python
   if field_name in pdf.forms[0].fields:
       pdf.forms[0].fields[field_name].value = value
   ```

2. **Check field types before setting values**
   ```python
   field = pdf.forms[0].fields[field_name]
   if field.field_type == "/Btn":  # Checkbox/Radio
       value = "/Yes" if checked else "/Off"
   elif field.field_type == "/Tx":  # Text
       value = str(value)
   ```

3. **Use correct checkbox values**
   ```python
   checkbox.value = "/Yes"   # ✅ Checked
   checkbox.value = "/Off"   # ✅ Unchecked
   ```

4. **Verify dropdown options**
   ```python
   if value in field.export_values:
       field.value = value
   ```

5. **Handle missing fields gracefully**
   ```python
   try:
       pdf.forms[0].fields[field_name].value = value
   except KeyError:
       print(f"Warning: Field '{field_name}' not found")
   ```

6. **Create backups before modifying**
   ```python
   import shutil
   shutil.copy2("original.pdf", "backup.pdf")
   ```

7. **Test with sample PDFs first**
   ```python
   # Always test with a copy
   test_fill_form("test_form.pdf", "test_output.pdf")
   ```

### DON'T ❌

1. **Don't assume field names**
   ```python
   # ❌ Wrong: assumes field exists
   pdf.forms[0].fields["name"].value = "John"

   # ✅ Correct: check first
   if "name" in pdf.forms[0].fields:
       pdf.forms[0].fields["name"].value = "John"
   ```

2. **Don't use boolean for checkboxes**
   ```python
   checkbox.value = True   # ❌ Wrong
   checkbox.value = "/Yes" # ✅ Correct
   ```

3. **Don't set invalid dropdown values**
   ```python
   field.value = "California"  # ❌ Wrong
   field.value = "CA"          # ✅ Correct (use code)
   ```

4. **Don't forget to save changes**
   ```python
   # ❌ Wrong: changes not saved
   pdf.forms[0].fields["name"].value = "John"

   # ✅ Correct: create writer and save
   writer = pypdf.PdfWriter()
   writer.add_pages(pdf.pages)
   writer.write("output.pdf")
   ```

5. **Don't modify original without backup**
   ```python
   # ❌ Risky: overwrites original
   fill_form("original.pdf", "original.pdf", data)

   # ✅ Safe: creates new file
   fill_form("original.pdf", "filled.pdf", data)
   ```

6. **Don't ignore encryption**
   ```python
   # ❌ Will fail on encrypted PDFs
   pdf = pypdf.PdfReader("encrypted.pdf")

   # ✅ Handle encryption
   if pdf.is_encrypted:
       pdf.decrypt(password)
   ```

## Testing Checklist

Before deploying form filling scripts, verify:

- [ ] Field names match PDF specification
- [ ] All required fields are identified
- [ ] Checkbox values use `/Yes` and `/Off`
- [ ] Dropdown values match export_values
- [ ] Date formats are consistent
- [ ] Phone number formats validated
- [ ] Email addresses validated
- [ ] Encrypted PDFs handled
- [ ] Backups created before modifications
- [ ] Output PDFs open correctly
- [ ] Filled values display properly
- [ ] Forms can be flattened if needed
- [ ] Error handling implemented
- [ ] Logging added for debugging

---

**Version**: 1.0.0
**Last Updated**: 2026-01-10
**Maintainer**: Doc Team
