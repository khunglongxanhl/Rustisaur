build:
	docker-compose build

run:
	docker-compose run --rm rustisaur rustisaur repl

run-file:
	docker-compose run --rm rustisaur rustisaur run /app/scripts/$(FILE)

up:
	docker-compose up -d

logs:
	docker-compose logs -f --tail=100

down:
	docker-compose down

shell:
	docker-compose run --rm --entrypoint /bin/bash rustisaur

clean:
	docker-compose down --rmi all --volumes --remove-orphans
	docker system prune -f