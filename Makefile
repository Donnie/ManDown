build:
	@echo "Building for prod"
	docker build -t donnieashok/mandown:prod .

up:
	@echo "Running for Prod"
	docker run -dit --rm -p 1338:8080 --name mandown donnieashok/mandown:prod

deploy: build
	docker push donnieashok/mandown:prod
	@echo "Deployed!"

clean:
	docker stop mandown
	@echo "all clear"
