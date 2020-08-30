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
	ssh root@vultr docker pull donnieashok/mandown:prod
	- ssh root@vultr docker stop mandown
	scp -r ./.env root@vultr:/root/
	ssh root@vultr docker run -d --restart on-failure -v /home/mandown/:/db/ --env-file /root/.env --name mandown donnieashok/mandown:prod
	ssh root@vultr rm /root/.env
	@echo "Is live"

publish: deploy live

clean:
	docker stop mandown
	@echo "all clear"
