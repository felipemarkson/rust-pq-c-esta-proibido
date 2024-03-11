FROM alpine:3.19
COPY target/x86_64-unknown-linux-musl/release/database /bin/database
COPY dockerfiles/database.sh /database.sh
RUN chmod +x /database.sh
ENTRYPOINT "./database.sh"