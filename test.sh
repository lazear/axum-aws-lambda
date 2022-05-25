cargo build
docker build . -t axum
docker run -p 9000:8080 axum

# curl -X POST -H "Content-Type: application/json" -d @test.json localhost:8080