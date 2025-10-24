#!/usr/bin/env sh

set -e
#python3 /app/backend/src/core/settings/logger/logg.py
#cp usr/tmp/__init__.py usr/local/lib/python3.8/site-packages/motor/frameworks/asyncio/__init__.py
cd /var/app/backend/src/prices/models
alembic upgrade head
cd /var/app/backend/src
gunicorn main:app --bind 0.0.0.0:$APP_PORT -w 4 -k uvicorn.workers.UvicornWorker