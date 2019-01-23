FROM ekidd/rust-musl-builder

# Add our source code.
COPY . ./

# Build our application.
RUN cargo build --release
