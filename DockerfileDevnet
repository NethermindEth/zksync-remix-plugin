FROM ubuntu:latest
ARG LATEST_RELEASE
WORKDIR /app
RUN apt-get update \
  && apt-get install -y curl jq \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

RUN echo "In memory node version: ${LATEST_RELEASE}"

# Download the release version specified by the build argument
RUN curl -s echo "https://api.github.com/repos/matter-labs/era-test-node/releases/tags/${LATEST_RELEASE}" | \
  jq -r '.assets[] | select(.name | contains("x86_64-unknown-linux-gnu")) | .browser_download_url' | \
  head -n 1 | xargs -I {} curl -L -o era_test_node.tar.gz {}

# Extract binary and make it executable
RUN tar xz -f era_test_node.tar.gz -C /usr/local/bin/
RUN chmod +x /usr/local/bin/era_test_node
EXPOSE 8011

# Run the binary
CMD ["era_test_node", "run"]
