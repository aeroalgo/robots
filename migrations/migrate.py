#!/usr/bin/env python3
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path

import psycopg2
from clickhouse_driver import Client
from pymongo import MongoClient

env_file = Path(__file__).parent.parent / "docker" / "env.local"
if env_file.exists():
    with open(env_file) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, value = line.split("=", 1)
                os.environ.setdefault(key, value)


class MigrationManager:
    def __init__(self, db_type, connection):
        self.db_type = db_type
        self.connection = connection
        self.migrations_dir = Path(__file__).parent / db_type

    def ensure_migration_table(self):
        if self.db_type == "postgres":
            cursor = self.connection.cursor()
            cursor.execute(
                """
                CREATE TABLE IF NOT EXISTS migration_history (
                    version INTEGER PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """
            )
            self.connection.commit()
            cursor.close()
        else:
            self.connection.execute(
                """
                CREATE TABLE IF NOT EXISTS migration_history (
                    version UInt32,
                    name String,
                    applied_at DateTime DEFAULT now()
                ) ENGINE = MergeTree()
                ORDER BY version
            """
            )

    def get_applied_migrations(self):
        if self.db_type == "postgres":
            cursor = self.connection.cursor()
            cursor.execute(
                "SELECT version FROM migration_history ORDER BY version"
            )
            applied = set(row[0] for row in cursor.fetchall())
            cursor.close()
            return applied
        else:
            result = self.connection.execute(
                "SELECT version FROM migration_history ORDER BY version"
            )
            return set(row[0] for row in result)

    def get_pending_migrations(self):
        applied = self.get_applied_migrations()
        all_migrations = []

        for file in sorted(self.migrations_dir.glob("*.sql")):
            try:
                version = int(file.stem.split("_")[0])
                if version not in applied:
                    all_migrations.append((version, file.name, file))
            except ValueError:
                print(f"Пропускаю файл с неверным форматом: {file.name}")

        return sorted(all_migrations, key=lambda x: x[0])

    def apply_migration(self, version, name, file_path):
        print(f"Применяю миграцию {version}: {name}")

        with open(file_path, "r", encoding="utf-8") as f:
            sql = f.read()

        if self.db_type == "postgres":
            cursor = self.connection.cursor()
            try:
                cursor.execute(sql)
                cursor.execute(
                    "INSERT INTO migration_history (version, name) VALUES (%s, %s)",
                    (version, name),
                )
                self.connection.commit()
                cursor.close()
                print(f"✓ Миграция {version} применена успешно")
            except Exception as e:
                self.connection.rollback()
                cursor.close()
                raise e
        else:
            try:
                for statement in sql.split(";"):
                    statement = statement.strip()
                    if statement:
                        self.connection.execute(statement)

                self.connection.execute(
                    "INSERT INTO migration_history (version, name) VALUES",
                    [(version, name)],
                )
                print(f"✓ Миграция {version} применена успешно")
            except Exception as e:
                raise e

    def run(self):
        print(f"\n=== Миграции {self.db_type.upper()} ===")
        self.ensure_migration_table()

        pending = self.get_pending_migrations()

        if not pending:
            print("Нет новых миграций для применения")
            return

        print(f"Найдено миграций для применения: {len(pending)}")

        for version, name, file_path in pending:
            try:
                self.apply_migration(version, name, file_path)
            except Exception as e:
                print(f"✗ Ошибка при применении миграции {version}: {e}")
                sys.exit(1)

        print(f"✓ Все миграции применены успешно\n")


def get_postgres_connection():
    return psycopg2.connect(
        host=os.getenv("POSTGRES_HOST", "localhost"),
        port=os.getenv("POSTGRES_PORT", "5432"),
        database=os.getenv("POSTGRES_DB", "trading"),
        user=os.getenv("POSTGRES_USER", "postgres"),
        password=os.getenv("POSTGRES_PASSWORD", "postgres"),
    )


def get_clickhouse_connection():
    return Client(
        host=os.getenv("CLICKHOUSE_HOST", "localhost"),
        port=int(os.getenv("CLICKHOUSE_NATIVE_PORT", "9002")),
        database=os.getenv("CLICKHOUSE_DB", "default"),
        user=os.getenv("CLICKHOUSE_USER", "default"),
        password=os.getenv("CLICKHOUSE_PASSWORD", ""),
    )


def get_mongodb_connection():
    host = os.getenv("MONGO_HOST", "localhost")
    port = int(os.getenv("MONGO_PORT", "27017"))
    user = os.getenv("MONGO_USER", "")
    password = os.getenv("MONGO_PASSWORD", "")
    database = os.getenv("MONGO_DATABASE", "trading_meta")

    if user and password:
        uri = f"mongodb://{user}:{password}@{host}:{port}/{database}?authSource=admin"
    else:
        uri = f"mongodb://{host}:{port}/{database}"

    return MongoClient(uri)


def apply_mongodb_migrations():
    print("\n=== Миграции MONGODB ===")
    migrations_dir = Path(__file__).parent / "mongodb"

    if not migrations_dir.exists():
        print("Папка миграций MongoDB не найдена")
        return

    mongo_client = get_mongodb_connection()
    db = mongo_client.get_database()

    if "migrations" not in db.list_collection_names():
        db.create_collection("migrations")

    applied = set()
    for doc in db.migrations.find():
        applied.add(doc["version"])

    pending = []
    for file in sorted(migrations_dir.glob("*.py")):
        try:
            version = int(file.stem.split("_")[0])
            if version not in applied:
                pending.append((version, file.name, file))
        except ValueError:
            print(f"Пропускаю файл: {file.name}")

    if not pending:
        print("Нет новых миграций для применения")
        mongo_client.close()
        return

    print(f"Найдено миграций для применения: {len(pending)}")

    for version, name, file_path in pending:
        print(f"Применяю миграцию {version}: {name}")
        try:
            import importlib.util

            spec = importlib.util.spec_from_file_location(
                "migration", file_path
            )
            migration_module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(migration_module)

            migration_module.apply_migration(db)

            db.migrations.insert_one(
                {
                    "version": version,
                    "name": name,
                    "applied_at": datetime.now(),
                }
            )
            print(f"✓ Миграция {version} применена успешно")
        except Exception as e:
            print(f"✗ Ошибка при применении миграции {version}: {e}")
            mongo_client.close()
            sys.exit(1)

    mongo_client.close()
    print("✓ Все миграции применены успешно\n")


def main():
    target = sys.argv[1] if len(sys.argv) > 1 else "all"

    if target in ["all", "postgres"]:
        try:
            print("Подключаюсь к PostgreSQL...")
            pg_conn = get_postgres_connection()
            pg_manager = MigrationManager("postgres", pg_conn)
            pg_manager.run()
            pg_conn.close()
        except Exception as e:
            print(f"Ошибка PostgreSQL: {e}")
            if target == "postgres":
                sys.exit(1)

    if target in ["all", "clickhouse"]:
        try:
            print("Подключаюсь к ClickHouse...")
            ch_conn = get_clickhouse_connection()
            ch_manager = MigrationManager("clickhouse", ch_conn)
            ch_manager.run()
            ch_conn.disconnect()
        except Exception as e:
            print(f"Ошибка ClickHouse: {e}")
            if target == "clickhouse":
                sys.exit(1)

    if target in ["all", "mongodb"]:
        try:
            print("Подключаюсь к MongoDB...")
            apply_mongodb_migrations()
        except Exception as e:
            print(f"Ошибка MongoDB: {e}")
            if target == "mongodb":
                sys.exit(1)

    print("✓ Миграции завершены успешно!")


if __name__ == "__main__":
    main()
