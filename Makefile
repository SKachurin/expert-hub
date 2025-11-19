.PHONY: dev prod down-dev down-prod logs-dev logs-prod build-prod

dev:
	docker-compose -f docker-compose.dev.yml --env-file .env.local up

prod:
	docker-compose -f docker-compose.prod.yml --env-file .env up -d

down-dev:
	docker-compose -f docker-compose.dev.yml down

down-prod:
	docker-compose -f docker-compose.prod.yml down

logs-dev:
	docker-compose -f docker-compose.dev.yml logs -f

logs-prod:
	docker-compose -f docker-compose.prod.yml logs -f

build-prod:
	docker-compose -f docker-compose.prod.yml build

db-shell-dev:
	docker-compose -f docker-compose.dev.yml exec db psql -U app_user -d app_db

db-shell-prod:
	docker-compose -f docker-compose.prod.yml exec db psql -U app_user -d app_db

migrate-dev:
	docker-compose -f docker-compose.dev.yml exec app sea-orm-cli migrate

migrate-prod:
	docker-compose -f docker-compose.prod.yml exec app sea-orm-cli migrate