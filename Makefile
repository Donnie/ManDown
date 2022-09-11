dev:
	goreload .

up:
	@echo "Running for Prod"
	docker run -dit --restart on-failure --name mandown donnieashok/mandown:prod

live:
	ssh donnie@mandown sudo docker pull donnieashok/mandown:prod
	- ssh donnie@mandown sudo docker stop mandown
	- ssh donnie@mandown sudo docker rm mandown
	ssh donnie@mandown 'mkdir -p ~/mandown/db'
	scp ./.env donnie@mandown:~/mandown/.env
	# scp ./db/db.csv donnie@mandown:~/mandown/db/db.csv
	ssh donnie@mandown 'sudo docker run -d --restart on-failure -v ~/mandown/db:/db/ --env-file ~/mandown/.env --name mandown donnieashok/mandown:prod'
	ssh donnie@mandown 'rm ~/mandown/.env'
	@echo "Is live"

publish: deploy live

test:
	go test ./...

clean:
	docker stop mandown
	@echo "all clear"
