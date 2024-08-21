FROM alpine as build

WORKDIR /app

COPY . .
RUN apk add openssl-dev musl-dev cargo
RUN cargo build -r

FROM alpine

LABEL org.opencontainers.image.source=https://github.com/NoseferatuWKF/interceder

WORKDIR /app

COPY --from=build /app/target/release/interceder .

RUN apk add libgcc

EXPOSE 3435

CMD ["./interceder"]
