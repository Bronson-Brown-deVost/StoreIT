# QR Codes & Print Labels

StoreIT can generate QR codes for any location, container, or item. Scanning a QR code opens the entity directly in StoreIT — useful for labeling shelves, bins, and storage areas.

## Generating QR Codes

On any location, container, or item detail page, tap the **Print Label** button. This opens a print-ready label that includes:

- The entity name
- Its location path (e.g., "Garage > Shelf A > Box 3")
- A QR code linking directly to that entity

## Printing Labels

The print label opens in a new window optimized for printing. Use your browser's print function (Ctrl+P / Cmd+P) to print it. Labels are designed to work well on:

- Standard label printers (e.g., Brother, Dymo)
- Regular paper (cut to size)
- Adhesive label sheets

## How QR Codes Work

Each QR code encodes a URL pointing to your StoreIT instance:

```
https://your-server/nfc/tag?uid=container-{uuid}
```

When scanned with any phone camera or QR reader, it opens StoreIT and navigates directly to that entity. If the user is not logged in, they'll be prompted to authenticate first.

## QR Codes vs NFC Tags

| Feature | QR Codes | NFC Tags |
|---------|----------|----------|
| Cost | Free (print on paper) | Requires NFC stickers |
| Scanning | Camera app | Hold phone near tag |
| Durability | Can fade/tear | Waterproof, durable |
| Range | Line of sight | ~2cm proximity |
| Setup | Print and stick | Write tag URI, then assign in app |

Both can be used together — put a QR code on the outside of a box and an NFC tag inside the lid.
