-- ============================================================================
--  03 · SEED  —  row data   (DML only)
-- ============================================================================
--  Populates the tables defined in 02-schema.sql. Kept separate so structure
--  and data never share a file. `\connect` state does NOT survive across initdb
--  files (each runs in its own psql session against POSTGRES_DB=publicdb), so
--  this file re-selects secretdb explicitly before seeding its rows.
-- ============================================================================

-- ----------------------------------------------------------------------------
--  publicdb.data  ·  innocuous rows (the app is meant to show these)
-- ----------------------------------------------------------------------------
INSERT INTO public.data (id, label) VALUES
    (1, 'публичная запись A'),
    (2, 'публичная запись B'),
    (3, 'публичная запись C');

-- ----------------------------------------------------------------------------
--  secretdb.data  ·  rows the app must never surface (injection target)
-- ----------------------------------------------------------------------------
\connect secretdb

INSERT INTO public.data (id, label) VALUES
    (1, 'API_KEY=sk-live-DO-NOT-LEAK-0001'),
    (2, 'db_superuser_password=postgres'),
    (3, 'recovery_phrase=correct horse battery staple');
