dev:
	cargo run

build:
	docker build -t donnieashok/mandown:rust .

up:
	@echo "Running for Prod"
	docker run -d -v ./db:/db/ --env-file .env --name mandown donnieashok/mandown:rust

clean:
	docker stop mandown
	docker rm mandown
	@echo "all clear"
