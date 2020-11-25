FROM ekidd/rust-musl-builder AS build

# Add our source code.
ADD --chown=rust:rust . ./

# Build our application.
RUN cargo build --release

FROM scratch

COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/envsub /bin/

ENTRYPOINT ["/bin/envsub"]
