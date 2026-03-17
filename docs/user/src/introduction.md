# Introduction

StoreIT is a self-hosted home inventory management system. It helps you track where everything in your home is stored — from tools in the garage to craft supplies in the closet.

## Key Features

- **Hierarchical organization** — Organize belongings as Locations > Containers > Items, with unlimited nesting
- **AI-powered identification** — Snap a photo and AI identifies the item, generating searchable descriptions automatically
- **NFC tag support** — Tap an NFC tag on a container to instantly see contents or move items
- **Full-text search** — Find items by name, description, or AI-generated tags
- **Progressive Web App** — Install on any phone or tablet; works offline
- **Single-binary deployment** — One file to run, no external web server needed
- **Multi-user** — Share inventory across family members with OIDC authentication

## How It Works

StoreIT uses a simple hierarchy:

```
Home (Location)
├── Kitchen (Sub-Location)
│   ├── Drawer Left (Container)
│   │   ├── Scissors
│   │   ├── Tape
│   │   └── Markers
│   └── Cabinet Top (Container)
│       └── Small Box (Sub-Container)
│           ├── Batteries
│           └── Light Bulbs
└── Garage (Sub-Location)
    └── Shelf A (Container)
        ├── Drill
        └── Screwdriver Set
```

**Locations** are physical spaces (rooms, buildings). **Containers** hold items and can be nested. **Items** are the things you're tracking.

Every entity can have photos attached. When you add an item by photo, the AI examines the image and suggests a name, description, and searchable tags — you just confirm or tweak.
