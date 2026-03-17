# NFC Tags

NFC (Near Field Communication) tags let you tap your phone on a physical container to instantly see its contents or move items.

## Setup

1. Get NFC sticker tags (NTAG213 or NTAG215 are common and inexpensive)
2. Use an NFC writing app to write a URL to the tag: `https://your-storeit-server/nfc/{tag-id}`
3. In StoreIT, navigate to the container and link the NFC tag via the container's settings

## Usage

### View Contents

Tap your phone on an NFC-tagged container. StoreIT opens and shows the container's contents.

### Move Items

1. Open an item and tap **Move**
2. Tap the NFC tag on the destination container
3. The item moves to that container

## Requirements

- A phone with NFC capability (most modern smartphones)
- NFC sticker tags
- StoreIT accessible via HTTPS (required for Web NFC API on some browsers)
