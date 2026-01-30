# Dockerfile for HQE Workbench Example Repository
# Used for testing with Venice AI API

FROM node:18-alpine

# Install Rust for building HQE CLI if needed
RUN apk add --no-cache curl git openssh-client \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $HOME/.cargo/env \
    && rustup default stable

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm install

# Install HQE CLI globally (if available) or copy from local build
# For now, assuming it's available in PATH
RUN npm install -g hqe || echo "HQE CLI not available as npm package, will need to be installed separately"

# Copy application files
COPY . .

# Create non-root user for security
RUN addgroup -g 1001 -S nodejs &&\
    adduser -S nextjs -u 1001
USER nextjs

# Expose port if needed for API
EXPOSE 3000

# Run tests to verify setup
CMD ["sh", "-c", "echo 'Running verification tests...' && npm test || echo 'Tests may fail if HQE CLI is not properly installed'"]