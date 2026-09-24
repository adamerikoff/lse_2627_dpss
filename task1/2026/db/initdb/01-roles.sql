-- ============================================================================
--  01 · ROLES  —  cluster-wide identity
-- ============================================================================
--  Runs first, as the superuser `postgres`. Roles are cluster-wide, so the
--  `usr` role created here is visible in every database that later files touch
--  (publicdb and secretdb alike). No database objects are created in this file.
-- ============================================================================

-- Store the password as a SCRAM-SHA-256 verifier (not legacy MD5), so the
-- CREATE ROLE ... PASSWORD below is hashed with the intended algorithm.
SET password_encryption = 'scram-sha-256';

CREATE ROLE usr
    LOGIN
    INHERIT
    NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD 'pass';
