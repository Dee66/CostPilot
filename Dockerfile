# CostPilot Docker Image
FROM alpine:latest

# Install ca-certificates for HTTPS downloads
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN addgroup -g 1000 costpilot && \
    adduser -D -s /bin/sh -u 1000 -G costpilot costpilot

# Set the working directory
WORKDIR /app

# Download and install CostPilot binary
# This will be replaced with the actual binary during build
COPY costpilot /usr/local/bin/costpilot
RUN chmod +x /usr/local/bin/costpilot

# Change ownership to non-root user
RUN chown costpilot:costpilot /usr/local/bin/costpilot

# Switch to non-root user
USER costpilot

# Verify installation
RUN costpilot --version

# Set the entrypoint
ENTRYPOINT ["costpilot"]

# Default command
CMD ["--help"]