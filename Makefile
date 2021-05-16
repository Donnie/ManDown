dev:
	goreload .

build:
	@echo "Building for prod"
	docker build -t donnieashok/mandown:prod .

up:
	@echo "Running for Prod"
	docker run -dit --restart on-failure --name mandown donnieashok/mandown:prod

deploy: build
	echo "$(DOCKER_PASSWORD)" | docker login -u "$(DOCKER_USERNAME)" --password-stdin
	docker push donnieashok/mandown:prod
	@echo "Deployed!"

live:
	ssh root@mandown docker pull donnieashok/mandown:prod
	- ssh root@mandown docker stop mandown
	- ssh root@mandown docker rm mandown
	scp -r ./.env root@mandown:/root/
	ssh root@mandown docker run -d --restart on-failure -v /home/mandown/:/db/ --env-file /root/.env --name mandown donnieashok/mandown:prod
	ssh root@mandown rm /root/.env
	@echo "Is live"

publish: deploy live

test:
	go test ./...

clean:
	docker stop mandown
	@echo "all clear"
