-- ============================================================================
--  02 · SCHEMA  —  databases, privileges, tables   (DDL + grants, NO data)
-- ============================================================================
--  Structure only. Every INSERT lives in 03-seed.sql, so DDL and DML never sit
--  mixed in one file. The demo hinges on two databases each owning a table with
--  the SAME name, `data`: the app runs an UNQUALIFIED `SELECT * FROM data`, so
--  the connection string alone decides which database — and which rows — it
--  resolves to.
-- ============================================================================

-- ----------------------------------------------------------------------------
--  publicdb  ·  the data the app is MEANT to show
-- ----------------------------------------------------------------------------
--  Connected here by default (POSTGRES_DB=publicdb). Revoke the implicit PUBLIC
--  privileges first, so the explicit grants to `usr` are the only ones loaded.
REVOKE ALL     ON DATABASE publicdb FROM PUBLIC;
GRANT  CONNECT ON DATABASE publicdb TO   usr;

REVOKE ALL   ON SCHEMA public FROM PUBLIC;
GRANT  USAGE ON SCHEMA public TO   usr;

CREATE TABLE public.data (
    id    integer PRIMARY KEY,
    label text    NOT NULL
);
GRANT SELECT ON public.data TO usr;

-- ----------------------------------------------------------------------------
--  secretdb  ·  the data the app NEVER points at
-- ----------------------------------------------------------------------------
--  `usr` is fully authorised here too; the ONLY thing keeping it out is the
--  app's intent to stay in publicdb. Injection defeats that intent, not any
--  GRANT. CREATE DATABASE cannot run inside a transaction block; the postgres
--  image runs initdb *.sql with autocommit (no --single-transaction), so this
--  is safe.
CREATE DATABASE secretdb;

\connect secretdb

REVOKE ALL     ON DATABASE secretdb FROM PUBLIC;
GRANT  CONNECT ON DATABASE secretdb TO   usr;

REVOKE ALL   ON SCHEMA public FROM PUBLIC;
GRANT  USAGE ON SCHEMA public TO   usr;

CREATE TABLE public.data (
    id    integer PRIMARY KEY,
    label text    NOT NULL
);
GRANT SELECT ON public.data TO usr;
