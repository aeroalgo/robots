#!/bin/bash

cd "$(dirname "$0")/.."

if [ ! -d "venv" ]; then
    echo "Виртуальное окружение не найдено. Создайте его: python -m venv venv"
    exit 1
fi

source venv/bin/activate

if [ -f "docker/env.local" ]; then
    export $(grep -v '^#' docker/env.local | xargs)
fi

python migrations/migrate.py "$@"

