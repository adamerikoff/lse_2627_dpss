-- Роли и права (п. 5). Выполняется первым, под суперпользователем postgres.
SET password_encryption = 'scram-sha-256';

-- Админ банка: владеет таблицами (см. 02), то есть может DDL — ALTER, DROP
-- CONSTRAINT, DROP TABLE — плюс любой DML.
CREATE ROLE bank_admin
    LOGIN INHERIT
    NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD 'admin_pass';

-- Сотрудник банка (оператор CBS): только SELECT / INSERT / UPDATE (см. гранты
-- в 02). Без DELETE и DDL. NOINHERIT + членство в bank_admin — заложенная
-- мисконфигурация: при обычном подключении прав админа нет, но инъекция
-- options='-c role=bank_admin' в строку подключения переключает роль сессии
-- без пароля админа (членство разрешает SET ROLE). Это демонстрирует уязвимая
-- версия входа; защищённая кладёт ввод в отдельное поле Config, инъекция гаснет.
CREATE ROLE bank_employee
    LOGIN NOINHERIT
    NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD 'employee_pass';
GRANT bank_admin TO bank_employee;

REVOKE ALL     ON DATABASE neobank FROM PUBLIC;
GRANT  CONNECT ON DATABASE neobank TO   bank_admin, bank_employee;

REVOKE ALL   ON SCHEMA public FROM PUBLIC;
GRANT  USAGE ON SCHEMA public TO   bank_admin, bank_employee;
