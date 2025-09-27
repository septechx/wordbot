#!/bin/sh

echo "Migrating database"

for migration in migrations/*; do
  echo "Executing $migration"
  sqlite3 wordbot.db "$(cat "$migration")"
done

echo "Done"
