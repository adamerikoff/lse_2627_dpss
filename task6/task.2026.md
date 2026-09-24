# Задание №6

## Хранение и доступ к секретам

Выбрать используемый вариант: HashiCorp Vault, Infisical, Kubernetes. Для осмысленного выбора можно ознакомиться с https://habr.com/ru/companies/bastion/articles/824748/

### Вариант 1:

1. Сделать копию репозитория задания 2.
2. Добавить официальный Docker-образ HashiCorp Vault (https://hub.docker.com/r/hashicorp/vault) в Docker Compose в режиме сервера для разработки (Running Vault in Server Mode for Development).
3. В файле Docker Compose настроить volume на внешний путь так, чтобы файл `/vault/file` хранился в volume и был также доступен вне образа `hashicorp/vault`.
4. Включить в Vault метод авторизации AppRole (см. https://habr.com/ru/articles/653927/#4).
5. Создать в Vault секрет (логин и пароль) для доступа к БД.
6. Создать политику для чтения этого секрета.
7. Создать AppRole для сервиса (pinger, heartbit) из задания 2.
8. Получить role-id для AppRole.
9. Добавить в код сервиса (pinger, heartbit) библиотеку-клиент для Vault (см. примеры https://github.com/orgs/hashicorp-education/repositories).
10. Настроить в коде клиент Vault, передавая из переменных среды окружения в клиент Vault параметры: сетевой адрес Vault, role-id и путь до секрета.
11. В коде с помощью клиента Vault получать секрет с логином и паролем к БД из Vault перед каждым запросом к БД.
12. Протестировать работу сервиса с правильными значениями секрета и неверными.

Дополнительные материалы:

- https://habr.com/ru/articles/653927/
- https://habr.com/ru/articles/536694/
- https://habr.com/ru/companies/jetinfosystems/articles/762194/
- https://hub.docker.com/r/hashicorp/vault
- https://github.com/orgs/hashicorp-education/repositories

### Вариант 2:

1. Сделать копию репозитория задания 2.
2. Добавить официальный Docker-образ Infisical (https://hub.docker.com/r/infisical/infisical) в Docker Compose.
3. В файле Docker Compose настроить volume’ы на внешние пути так, чтобы папки с содержимым `/var/lib/postgresql/data` и `/data` хранились в volume’ах и были также доступны вне образа `infisical/infisical`. Пример в https://github.com/Infisical/infisical/blob/main/docker-compose.prod.yml и в https://infisical.com/blog/self-hosting-infisical-homelab
4. Создать проект для приложения pinger / heartbit.
5. Создать в проекте секрет (логин и пароль) для доступа к БД сервиса (pinger, heartbit) из задания 2.
6. Настроить Universal Auth для будущего клиента Infisical и получить Client-Id и Client-Secret, подробнее: https://infisical.com/docs/documentation/platform/identities/universal-auth
7. Добавить identity, созданную на предыдущем шаге, в проект, созданный на 4 шаге.
8. Добавить в код сервиса (pinger, heartbit) SDK для Infisical (см. https://infisical.com/docs/sdks/overview). Если SDK для вашего языка недоступен, использовать метод wrap с помощью `infisical run` (см. https://infisical.com/docs/cli/usage#feed-secrets-to-your-application).
9. Настроить в коде SDK Infisical, передавая из переменных среды окружения в клиент параметры: сетевой адрес Infisical, Client-Id и Client-Secret.
10. В коде с помощью SDK Infisical получать секрет с логином и паролем к БД из Infisical перед каждым запросом к БД.
11. Протестировать работу сервиса с правильными значениями секрета и неверными.

Дополнительные материалы:

- https://desoft.ru/2024/12/27/platforma-dlya-upravleniya-sekretami-infisical/
- https://telegra.ph/Upravlenie-sekretami-Ansible-s-pomoshchyu-Infisical-09-30
- https://github.com/Infisical/infisical-dotnet-configuration

### Вариант 3:

Этот вариант подходит только тем, у кого есть настроенный Kubernetes или кто готов его развернуть самостоятельно.

1. Сделать копию репозитория задания 2.
2. Установить и настроить Kubernetes Secrets.
3. Настроить шифрование секретов Kubernetes Secrets в состоянии покоя, используя secretbox, детали: https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/
4. Создать секреты (логин и пароль) для доступа к БД PostgreSQL из задания 2.
5. Обеспечить корректную передачу секретов из Kubernetes Secrets в сервис (pinger, heartbit).
6. Протестировать работу сервиса с правильными значениями секрета и неверными.

## Требования для сдачи:

1. Расположить исходный код программы, Dockerfile, Env-файл, Docker Compose-файл в отдельный публичный репозиторий или в отдельную ветку предыдущего репозитория.
2. Продемонстрировать работу приложения в случае верного значения секрета от БД и подключения к БД, в случае неправильного значения секрета от БД.
3. Ответить на вопросы преподавателя.
