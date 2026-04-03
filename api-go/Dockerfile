FROM golang:1.22-alpine AS build
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /out/api-go .

FROM gcr.io/distroless/static:nonroot
WORKDIR /
COPY --from=build /out/api-go /api-go
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/api-go"]

