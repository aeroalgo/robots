# include ../config/.env # for local development
# export YC_SECRET_ID=e6qva3oidh68mg7jdajk



.PHONY: mt5
mt5:
	WINEPREFIX="/media/aero/Z/Games/mt5" python -m mt5linux C:/users/aero/AppData/Local/Programs/Python/Python38/python.exe

.PHONY: sync_data.quick
sync_data.quick:
	cd algo_robots && python sync_historical_data.py --quick

.PHONY: g
g:
	git add --all
	git commit -m "x"
	git push


.PHONY: lint
lint:
	ruff app/ core/ tests/ --verbose


.PHONY: test
test:
	pytest


.PHONY: migration
migration:
	alembic revision --autogenerate -m ${name}


.PHONY: migration.manual
migration.manual:
	alembic revision -m ${name}


.PHONY: migrate
migrate:
	cd migrations && python migrate.py all


.PHONY: downgrade
downgrade:
	alembic downgrade ${revision}


.PHONY: stamp
stamp:
	alembic stamp base