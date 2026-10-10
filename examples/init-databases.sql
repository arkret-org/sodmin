-- Keep service-owned migrations in separate databases, including on restart.
SELECT 'CREATE DATABASE coauth'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'coauth') \gexec
SELECT 'CREATE DATABASE soland'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'soland') \gexec
