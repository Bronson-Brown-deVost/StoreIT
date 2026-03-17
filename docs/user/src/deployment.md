# Deployment

## Single Binary

StoreIT compiles to a single binary with the frontend embedded. No separate web server is needed.

```bash
./storeit-server
```

By default, the database is created at `./data/storeit.db` and images are stored in `./data/images/`. See [Configuration](./configuration.md) for all options.

### Systemd Service

```ini
[Unit]
Description=StoreIT Inventory Management
After=network.target

[Service]
Type=simple
User=storeit
WorkingDirectory=/opt/storeit
ExecStart=/opt/storeit/storeit-server
EnvironmentFile=/opt/storeit/.env
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Docker Compose (Recommended for Docker)

A ready-to-use `docker-compose.yml` and `.env.docker` template are included in the repository:

```bash
mkdir storeit && cd storeit

# Download compose file and env template
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/docker-compose.yml
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/.env.docker

# Create your .env from the template — at minimum, change STOREIT_SESSION_SECRET
cp .env.docker .env
nano .env

docker compose up -d
```

Open http://localhost:8080. Default login: `admin` / `changeme`.

### Data Storage

The compose file uses **bind mounts** by default, so your data lives directly on the host filesystem where you can see, back up, and manage it:

```
./data/
  db/storeit.db          # SQLite database
  images/                # Uploaded photos and thumbnails
    ab/ab1234...jpg
    ab/ab1234..._thumb.webp
```

To change where data is stored, edit the volume paths in `docker-compose.yml`:

```yaml
volumes:
  - /mnt/nas/storeit/db:/data/db
  - /mnt/nas/storeit/images:/data/images
```

If you prefer Docker named volumes instead (opaque, managed by Docker):

```yaml
volumes:
  - storeit-db:/data/db
  - storeit-images:/data/images

# Add at the bottom of the file:
volumes:
  storeit-db:
  storeit-images:
```

### Docker Run

If you prefer a single command:

```bash
mkdir -p ./data/db ./data/images

docker run -d \
  --name storeit \
  -p 8080:8080 \
  -v ./data/db:/data/db \
  -v ./data/images:/data/images \
  --env-file .env \
  ghcr.io/bronson-brown-devost/storeit:latest
```

### Building Locally

Build the binary first, then create the Docker image:

```bash
git clone https://github.com/Bronson-Brown-deVost/StoreIT.git
cd StoreIT
make build-all
docker build --build-arg BINARY=./target/release/storeit-server -t storeit .
docker run -d --name storeit -p 8080:8080 -v ./data/db:/data/db -v ./data/images:/data/images --env-file .env storeit
```

## Docker Image Details

- **Registry**: `ghcr.io/bronson-brown-devost/storeit`
- **Tags**: `latest` and version-specific (e.g., `0.1.0`)
- **Architectures**: `linux/amd64` and `linux/arm64`
- **Base image**: Alpine Linux
- **Data paths**: `/data/db` (database) and `/data/images` (photos)
- **Port**: `8080`

## Reverse Proxy

For production, put StoreIT behind a reverse proxy for TLS termination.

### Caddy

```
storeit.example.com {
    reverse_proxy localhost:8080
}
```

### Nginx

```nginx
server {
    listen 443 ssl;
    server_name storeit.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    client_max_body_size 50M;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Production Checklist

1. Set `STOREIT_SESSION_SECRET` to a random 32+ character string
2. Configure OIDC authentication (or change the default admin password)
3. Set `STOREIT_AUTH_REDIRECT_URI` to your public callback URL
4. Set `STOREIT_ANTHROPIC_API_KEY` for AI identification
5. Ensure database and image paths point to persistent storage
6. Set up TLS via reverse proxy
7. Set up regular backups (see [Backup & Restore](./backup.md))
