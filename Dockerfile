# Minimal runtime image — binary is pre-built by CI and passed via build arg.
# For local builds, use: docker build --build-arg BINARY=./target/release/storeit-server .
FROM alpine:3.21

ARG BINARY=storeit-server
COPY ${BINARY} /usr/local/bin/storeit-server
RUN chmod +x /usr/local/bin/storeit-server

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

RUN apk add --no-cache ca-certificates && \
    mkdir -p /data/db /data/images

ENV DATABASE_URL=sqlite:/data/db/storeit.db?mode=rwc
ENV STOREIT_IMAGE_PATH=/data/images
ENV STOREIT_BIND=0.0.0.0:8080

EXPOSE 8080
VOLUME /data

ENTRYPOINT ["docker-entrypoint.sh"]
