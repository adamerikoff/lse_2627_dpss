-- Схема neobank (п. 2, 3):
--   customers 1──N accounts 1──N transactions
--                  accounts 1──N cards

CREATE TABLE customers (
    id         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    full_name  text        NOT NULL,
    email      text        NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE accounts (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    customer_id    bigint        NOT NULL REFERENCES customers(id),
    account_number text          NOT NULL UNIQUE,
    currency       char(3)       NOT NULL DEFAULT 'USD',
    -- Главное ограничение демо: счёт не уходит в минус. Клиент не может
    -- нарушить его напрямую; эскалированная до bank_admin сессия снимает его
    -- через ALTER TABLE ... DROP CONSTRAINT (Tampering, задание 4).
    balance        numeric(14,2) NOT NULL DEFAULT 0 CONSTRAINT balance_non_negative CHECK (balance >= 0),
    status         text          NOT NULL DEFAULT 'active'
                   CHECK (status IN ('active', 'frozen', 'closed')),
    opened_at      timestamptz   NOT NULL DEFAULT now()
);

CREATE TABLE transactions (
    id         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id bigint        NOT NULL REFERENCES accounts(id),
    kind       text          NOT NULL
               CHECK (kind IN ('deposit', 'withdrawal', 'fee', 'transfer_in', 'transfer_out')),
    amount     numeric(14,2) NOT NULL,
    note       text,
    created_at timestamptz   NOT NULL DEFAULT now()
);

CREATE TABLE cards (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id  bigint NOT NULL REFERENCES accounts(id),
    card_number text   NOT NULL UNIQUE,
    holder_name text   NOT NULL,
    expires_on  date   NOT NULL DEFAULT (CURRENT_DATE + INTERVAL '4 years'),
    status      text   NOT NULL DEFAULT 'active'
                CHECK (status IN ('active', 'blocked', 'expired'))
);

-- Внешние ключи.
CREATE INDEX ON accounts(customer_id);
CREATE INDEX ON transactions(account_id);
CREATE INDEX ON cards(account_id);

-- Колонки фильтрации (card_number уже под UNIQUE).
CREATE INDEX ON accounts(currency);
CREATE INDEX ON accounts(balance);
CREATE INDEX ON accounts(status);
CREATE INDEX ON transactions(kind);
CREATE INDEX ON transactions(amount);
CREATE INDEX ON cards(holder_name);

-- Владелец — bank_admin: только владелец делает DDL (ALTER, DROP CONSTRAINT,
-- DROP TABLE). Скрипт идёт под postgres, поэтому передаём владение явно.
ALTER TABLE customers    OWNER TO bank_admin;
ALTER TABLE accounts     OWNER TO bank_admin;
ALTER TABLE transactions OWNER TO bank_admin;
ALTER TABLE cards        OWNER TO bank_admin;

-- Сотрудник: SELECT / INSERT / UPDATE, без DELETE и без DDL (п. 5).
GRANT SELECT, INSERT, UPDATE ON customers, accounts, transactions, cards TO bank_employee;
