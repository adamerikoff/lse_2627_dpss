-- Демонстрационные данные (п. 4). id присваивает сервер (IDENTITY),
-- строки связываем подзапросами по естественным ключам.
-- Четыре студентки миссии Ивакура, у каждой по два счёта.

INSERT INTO customers (full_name, email) VALUES
    ('Цуда Умэко',       'tsuda@example.com'),
    ('Ямакава Сутэмацу', 'yamakawa@example.com'),
    ('Нагаи Сигэко',     'nagai@example.com'),
    ('Ёсимасу Рёко',     'yoshimasu@example.com');

INSERT INTO accounts (customer_id, account_number, currency, balance, status) VALUES
    ((SELECT id FROM customers WHERE email = 'tsuda@example.com'),     'ACC-0001', 'JPY',  150000.00, 'active'),
    ((SELECT id FROM customers WHERE email = 'tsuda@example.com'),     'ACC-0002', 'USD',    1200.00, 'active'),
    ((SELECT id FROM customers WHERE email = 'yamakawa@example.com'),  'ACC-0003', 'KRW', 2000000.00, 'active'),
    ((SELECT id FROM customers WHERE email = 'yamakawa@example.com'),  'ACC-0004', 'EUR',     800.00, 'active'),
    ((SELECT id FROM customers WHERE email = 'nagai@example.com'),     'ACC-0005', 'CNY',    8000.00, 'frozen'),
    ((SELECT id FROM customers WHERE email = 'nagai@example.com'),     'ACC-0006', 'JPY',   50000.00, 'active'),
    ((SELECT id FROM customers WHERE email = 'yoshimasu@example.com'), 'ACC-0007', 'TWD',   30000.00, 'active'),
    ((SELECT id FROM customers WHERE email = 'yoshimasu@example.com'), 'ACC-0008', 'KRW',  500000.00, 'closed');

INSERT INTO cards (account_id, card_number, holder_name, expires_on, status) VALUES
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0001'), '4111-0000-0000-0001', 'UMEKO TSUDA',        '2028-05-31', 'active'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0002'), '4111-0000-0000-0002', 'UMEKO TSUDA',        '2027-03-31', 'active'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0003'), '4111-0000-0000-0003', 'SUTEMATSU YAMAKAWA', '2027-11-30', 'active'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0004'), '4111-0000-0000-0004', 'SUTEMATSU YAMAKAWA', '2026-08-31', 'blocked'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0005'), '4111-0000-0000-0005', 'SHIGEKO NAGAI',      '2026-02-28', 'expired'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0006'), '4111-0000-0000-0006', 'SHIGEKO NAGAI',      '2029-01-31', 'active'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0007'), '4111-0000-0000-0007', 'RYOKO YOSHIMASU',    '2028-12-31', 'active'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0008'), '4111-0000-0000-0008', 'RYOKO YOSHIMASU',    '2025-06-30', 'expired');

-- По три транзакции на счёт: пополнение, банкомат, ещё одна.
INSERT INTO transactions (account_id, kind, amount, note) VALUES
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0001'), 'deposit',     150000.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0001'), 'withdrawal',   -8000.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0001'), 'fee',            -500.00, 'обслуживание'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0002'), 'deposit',       1500.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0002'), 'withdrawal',    -300.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0002'), 'transfer_in',    200.00, 'перевод'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0003'), 'deposit',    2000000.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0003'), 'withdrawal',  -50000.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0003'), 'fee',           -2000.00, 'обслуживание'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0004'), 'deposit',       1000.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0004'), 'withdrawal',    -150.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0004'), 'transfer_out',   -50.00, 'перевод'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0005'), 'deposit',       8000.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0005'), 'withdrawal',    -600.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0005'), 'fee',            -120.00, 'обслуживание'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0006'), 'deposit',      50000.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0006'), 'withdrawal',   -4000.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0006'), 'transfer_in',   3000.00, 'перевод'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0007'), 'deposit',      30000.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0007'), 'withdrawal',   -2000.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0007'), 'fee',            -300.00, 'обслуживание'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0008'), 'deposit',     500000.00, 'первое пополнение'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0008'), 'withdrawal',  -25000.00, 'банкомат'),
    ((SELECT id FROM accounts WHERE account_number = 'ACC-0008'), 'transfer_out', -1000.00, 'перевод');
