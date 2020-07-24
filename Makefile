build:
	@echo "Building for prod"
	docker build -t donnieashok/mandown:prod .

up:
	@echo "Running for Prod"
	docker run -dit --rm -p 1338:8080 --name mandown donnieashok/mandown:prod

deploy: build
	docker push donnieashok/mandown:prod
	@echo "Deployed!"

live:
	ssh root@45.77.148.205 docker pull donnieashok/mandown:prod
	- ssh root@45.77.148.205 docker stop mandown
	scp -r ./.env root@45.77.148.205:/root/
	ssh root@45.77.148.205 docker run -d -v /home/mandown/:/db/ --rm --env-file /root/.env -p 1338:8080 --name mandown donnieashok/mandown:prod
	ssh root@45.77.148.205 rm /root/.env
	@echo "Is live"

publish: deploy live

clean:
	docker stop mandown
	@echo "all clear"
