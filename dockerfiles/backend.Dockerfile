FROM alpine:3.19
COPY target/x86_64-unknown-linux-musl/release/backend /bin/backend
COPY dockerfiles/backend.sh /backend.sh
RUN chmod +x /backend.sh
ENTRYPOINT "./backend.sh"