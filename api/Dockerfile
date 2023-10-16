# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libgmp3-dev

RUN git config --global url."https://github".insteadOf ssh://git@github

# Install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust binaries to the PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install the stable version of Rust
RUN rustup install stable

# Set the working directory
WORKDIR /app

# Copy the existing Rust application
COPY . .

RUN cargo build --release

EXPOSE 8000:80
CMD ["cargo", "run", "--release"]