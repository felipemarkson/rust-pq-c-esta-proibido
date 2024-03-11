FROM alpine:3.19
COPY target/x86_64-unknown-linux-musl/release/httpserver /bin/httpserver
COPY dockerfiles/httpserver.sh /httpserver.sh
RUN chmod +x /httpserver.sh
ENTRYPOINT "./httpserver.sh"