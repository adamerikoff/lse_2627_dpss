SET password_encryption = 'scram-sha-256';

CREATE ROLE usr
    LOGIN
    INHERIT
    NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD 'pass';

-- USAGE is not a database-level privilege in the GRANT grammar; CONNECT is its
-- equivalent. Revoking from PUBLIC first makes the grant actually load-bearing.
REVOKE ALL ON DATABASE task1db FROM PUBLIC;
GRANT CONNECT ON DATABASE task1db TO usr;

REVOKE ALL ON SCHEMA public FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO usr;
