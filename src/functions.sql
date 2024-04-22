-- All these functions are created on the pg_temp schema,
-- so they only last for the length of the connection

-- Simple test function
CREATE FUNCTION pg_temp.testfunc() 
RETURNS text AS
$$
  SELECT 'hello'::text
$$
LANGUAGE sql;

GRANT ALL ON SCHEMA public TO telegraf;

-- Give user access to all future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public
GRANT ALL ON TABLES TO telegraf;

-- Give user access to all future sequences
ALTER DEFAULT PRIVILEGES IN SCHEMA public
GRANT ALL ON SEQUENCES TO telegraf;

-- Give access to the db
GRANT SELECT ON ALL TABLES IN SCHEMA public TO grafana;
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO grafana;

-- Give user access to all future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public
GRANT SELECT ON TABLES TO grafana;

-- Give user access to all future sequences
ALTER DEFAULT PRIVILEGES IN SCHEMA public
GRANT SELECT ON SEQUENCES TO grafana;
