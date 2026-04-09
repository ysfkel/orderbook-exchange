
export DATABASE_URL="postgres://yusufkelo@localhost/markets"
sqlx database create


list databases
psql -d postgres -c '\l'

running migration 
> sqlx migrate add rename_symbol_to_name
> sqlx migrate run