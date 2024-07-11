FROM alpine as build

WORKDIR /app

COPY . .
RUN apk add openssl-dev musl-dev cargo
RUN cargo build -r

FROM alpine

WORKDIR /app

COPY --from=build /app/target/release/interceder .

RUN apk add libgcc

EXPOSE 42069

CMD ["./interceder"]
