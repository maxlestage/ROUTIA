# Le site de Routia : bundle WebAssembly + serveur Paquets, dans une image légère.
# docker build -t routia site && docker run -p 3000:3000 routia
FROM rust:1.99-bookworm AS build
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.130 --locked
WORKDIR /site
COPY . .
RUN ./build.sh

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /site/target/release/routia-site /usr/local/bin/routia-site
COPY --from=build /site/public ./public
COPY --from=build /site/pkg ./pkg
ENV PORT=3000
EXPOSE 3000
CMD ["routia-site"]
