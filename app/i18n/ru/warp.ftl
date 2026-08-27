# Zap Desktop — Русский
# Этот файл содержит русскую локализацию UI Zap. Отсутствующие ключи fallback-ятся на en/warp.ftl.
# Секции и ключи синхронизированы с исходной локалью.
#
# Формат ключей: kebab-case, префикс — surface, например settings-ai-title / drive-folder-rename-title.
# Интерполяция переменных использует синтаксис Fluent { $name }.

# =============================================================================
# SECTION: common (Owner: foundation)
# =============================================================================

app-name = Zap
app-tagline = Локально-ориентированный агентный терминал для разработчиков

common-ok = OK
common-cancel = Отмена
common-apply = Применить
common-save = Сохранить
common-delete = Удалить
common-confirm = Подтвердить
common-close = Закрыть
common-reset = Сбросить
common-back = Назад
common-next = Далее
common-yes = Да
common-no = Нет
common-continue = Продолжить
common-approve = Одобрить
common-deny = Отклонить
common-import = Импортировать
common-upgrade = Улучшить
common-default = По умолчанию
common-editing = Редактирование
common-viewing = Просмотр
common-tooltip-enter-edit-mode = Нажмите, чтобы начать редактирование
common-tooltip-exit-edit-mode = Нажмите, чтобы выйти из режима редактирования
common-restored = Восстановлено
common-continued = Продолжено
common-deleted = Удалено
common-send-feedback = Отправить отзыв
common-something-went-wrong = Что-то пошло не так
common-no-results-found = Ничего не найдено.
common-edit = Изменить
common-add = Добавить
common-remove = Удалить
common-rename = Переименовать
common-copy = Копировать
common-paste = Вставить
common-search = Поиск
common-view = Просмотр
common-loading = Загрузка…
common-error = Ошибка
common-warning = Предупреждение
common-info = Информация
common-success = Успешно
common-all = Все
common-none = Нет
common-unknown = Неизвестно
common-open = Открыть
common-restore = Восстановить
common-duplicate = Дублировать
common-export = Экспортировать
common-trash = Корзина
common-copy-link = Копировать ссылку
common-untitled = Без названия
common-retry = Повторить
common-maximize = Развернуть
common-discard = Отклонить
common-undo = Отменить
common-commit = Commit
common-push = Push
common-publish = Опубликовать
common-create = Создать
common-configure = Настроить
common-dismiss = Скрыть
common-manage = Управлять
common-failed = Не удалось
common-done = Готово
common-working = Выполняется
common-cut = Вырезать
common-previous = Предыдущий
common-suggested = Предлагаемое
common-copied-to-clipboard = Скопировано в буфер обмена
common-new = Новый
common-no-results = Нет результатов
common-learn-more = Подробнее
common-skip = Пропустить
common-get-warping = Начните работу в Zap
common-try-again = Попробовать снова
common-settings = Настройки
common-recommended = Рекомендуемое
common-enabled = Включено
common-disabled = Отключено
common-free = Бесплатно
common-list-prefix = {" - "}
common-current-directory = текущая директория

# =============================================================================
# SECTION: agent-management (Owner: agent-i18n-remaining)
# Files: app/src/ai/agent_management/**
# =============================================================================

agent-management-filter-all-tooltip = Показать ваши задачи агентов и все общие задачи команды
agent-management-filter-personal = Личные
agent-management-filter-personal-tooltip = Показать созданные вами задачи агентов
agent-management-get-started = Начать работу
agent-management-view-agents = Показать агентов
agent-management-clear-filters = Очистить фильтры
agent-management-clear-all = Очистить все
agent-management-new-agent = Новый агент
agent-management-status = Статус
agent-management-source = Источник
agent-management-created-on = Дата создания
agent-management-has-artifact = Есть артефакт
agent-management-harness = Harness
agent-management-environment = Окружение
agent-management-created-by = Автор
agent-management-last-24-hours = Последние 24 часа
agent-management-past-3-days = Последние 3 дня
agent-management-last-week = Последняя неделя
agent-management-artifact-pull-request = Pull Request
agent-management-artifact-plan = План
agent-management-artifact-screenshot = Скриншот
agent-management-artifact-file = Файл
agent-management-source-scheduled = По расписанию
agent-management-source-local-agent = Zap (локальный агент)
agent-management-source-cloud-agent = Агент Zap
agent-management-source-oz-web = Oz
agent-management-source-github-action = GitHub Action
agent-management-no-session-available = Нет доступной сессии
agent-management-session-expired = Сессия истекла
agent-management-session-expired-tooltip = Сессии истекают через неделю, и их больше нельзя открыть.
agent-management-metadata-source = Источник: { $source }
agent-management-metadata-harness = Harness: { $harness }
agent-management-metadata-run-time = Время выполнения: { $run_time }
agent-management-metadata-credits-used = Использовано кредитов: { $usage }
agent-management-environment-selected = Окружение: { $environment }
agent-management-loading-cloud-runs = Загрузка запусков агентов

# =============================================================================
# SECTION: workspace-runtime (Owner: agent-i18n-remaining)
# Files: app/src/workspace/view.rs
# =============================================================================

workspace-menu-update-warp-manually = Обновить Zap вручную
workspace-menu-whats-new = Что нового
workspace-menu-settings = Настройки
workspace-menu-keyboard-shortcuts = Сочетания клавиш
workspace-menu-documentation = Документация
workspace-menu-feedback = Обратная связь
workspace-menu-view-warp-logs = Показать логи Zap
workspace-menu-slack = Slack
workspace-toast-failed-load-conversation = Не удалось загрузить диалог.
workspace-toast-failed-load-conversation-for-forking = Не удалось загрузить диалог для ветвления.
workspace-toast-conversation-forking-failed = Не удалось выполнить ветвление диалога.
workspace-toast-no-terminal-pane-for-context = Нет открытой панели терминала. Откройте новую панель, чтобы прикрепить ее как контекст.
workspace-toast-plan-already-in-context = Этот план уже добавлен в контекст.
workspace-toast-command-still-running = Команда в этой сессии все еще выполняется.
workspace-toast-cannot-open-terminal-session = Не удалось открыть новую сессию терминала
workspace-toast-out-of-ai-credits = Похоже, у вас закончились кредиты AI.
workspace-toast-upgrade-more-credits = Улучшите тариф, чтобы получить больше кредитов.
workspace-toast-disabled-synchronized-inputs = Синхронизация ввода отключена во всех сессиях.
workspace-toast-conversation-deleted = Диалог удален
workspace-search-repos-placeholder = Поиск по репозиториям
workspace-search-tabs-placeholder = Поиск по вкладкам…
terminal-onekey-search-placeholder = Поиск сохраненных учетных данных SSH…
terminal-onekey-search-no-results = Нет подходящих учетных данных SSH
workspace-rearrange-toolbar-items = Изменить порядок элементов панели инструментов
workspace-new-session-agent = Агент
workspace-new-session-terminal = Терминал
workspace-new-session-cloud-oz = Вкладка агента
workspace-new-session-local-docker-sandbox = Локальная песочница Docker
workspace-new-worktree-config = Новая конфигурация worktree
workspace-new-tab-config = Новая конфигурация вкладки
workspace-reopen-closed-session = Повторно открыть закрытую сессию
app-menu-new-window = Новое окно
app-menu-save-new = Сохранить как новую…
app-menu-launch-configurations = Конфигурации запуска
app-menu-warp = Zap
app-menu-preferences = Настройки
app-menu-privacy-policy = Политика конфиденциальности…
app-menu-debug = Отладка
app-menu-set-default-terminal = Сделать Zap терминалом по умолчанию
app-menu-file = Файл
app-menu-edit = Правка
app-menu-use-warp-prompt = Использовать приглашение Zap
app-menu-copy-on-select-terminal = Копировать при выделении в терминале
app-menu-synchronize-inputs = Синхронизировать ввод
app-menu-view = Вид
app-menu-toggle-mouse-reporting = Переключить отчеты мыши
app-menu-toggle-scroll-reporting = Переключить отчеты прокрутки
app-menu-toggle-focus-reporting = Переключить отчеты фокуса
app-menu-compact-mode = Компактный режим
app-menu-tab = Вкладка
app-menu-ai = AI
app-menu-blocks = Блоки
app-menu-drive = Drive
app-menu-show-in-band-command-blocks = Показывать блоки команд in-band
app-menu-hide-in-band-command-blocks = Скрывать блоки команд in-band
app-menu-show-warpified-ssh-blocks = Показывать SSH-блоки Warpified
app-menu-hide-warpified-ssh-blocks = Скрывать SSH-блоки Warpified
app-menu-show-initialization-block = Показывать блок инициализации
app-menu-hide-initialization-block = Скрывать блок инициализации
app-menu-window = Окно
app-menu-enable-shell-debug-mode = Включить режим отладки shell (-x) для новых сессий
app-menu-disable-shell-debug-mode = Отключить режим отладки shell (-x) для новых сессий
app-menu-enable-pty-recording = Включить режим записи PTY (warp.pty.recording)
app-menu-disable-pty-recording = Отключить режим записи PTY (warp.pty.recording)
app-menu-enable-in-band-generators = Включить генераторы in-band для новых сессий
app-menu-disable-in-band-generators = Отключить генераторы in-band для новых сессий
app-menu-manually-toggle-network-status = Переключить статус сети вручную
app-menu-export-default-settings-csv = Экспортировать настройки по умолчанию в формате CSV в домашнюю директорию
app-menu-create-anonymous-user = Создать анонимного пользователя
app-menu-send-feedback = Отправить отзыв…
app-menu-help = Справка
app-menu-warp-documentation = Документация Zap…
app-menu-github-issues = GitHub Issues…
app-menu-warp-slack-community = Сообщество Zap в Slack…
workspace-update-and-relaunch-warp = Обновить и перезапустить Zap
workspace-updating-to-version = Обновление до ({ $version })
workspace-update-warp-manually = Обновить Zap вручную
pane-get-started-title = Начало работы
pane-new-tab-title = Новая вкладка

# =============================================================================
# SECTION: terminal-runtime (Owner: agent-i18n-remaining)
# Files: app/src/terminal/view.rs
# =============================================================================

terminal-banner-completions-not-working-prefix = Похоже, автодополнение не работает (
terminal-banner-more-info-lower = подробнее
terminal-banner-more-info = Подробнее
terminal-banner-completions-not-working-middle = ). Включение warpification для tmux в {" "}
terminal-banner-settings = настройках
terminal-banner-completions-not-working-suffix =  может решить эту проблему.
terminal-banner-shell-config-incompatible = Ваша конфигурация shell несовместима с Zap…{"  "}
terminal-banner-did-you-intend = Вы хотели {" "}
terminal-banner-move-cursor =  переместить курсор?
terminal-toast-powershell-subshells-not-supported = Вложенные shell PowerShell не поддерживаются
terminal-dont-ask-again = Больше не спрашивать
terminal-clear-upload = Очистить загрузку
terminal-manage-defaults = Управлять настройками по умолчанию
terminal-free-credits = Бесплатные кредиты
terminal-cloud-agent-run = Запуск агента
terminal-agent-header-for-terminal = для терминала
ssh-remote-choice-title = Выберите режим работы для этой удаленной сессии:
ssh-remote-choice-install-extension = Установить SSH-расширение Zap
ssh-remote-choice-install-extension-desc = Установите расширение Zap, чтобы в этой сессии стали доступны функции агента: просмотр файлов, ревью кода и интеллектуальное автодополнение команд.
ssh-remote-choice-continue-without-installing = Продолжить без установки
ssh-remote-choice-continue-without-installing-desc = Вы все равно получите возможности Warpified, но без функций агента.
ssh-remote-choice-manage-warpify-settings = Управлять настройками Warpify
ai-document-show-version-history = Показать историю версий
ai-document-update-agent = Обновить агента
ai-document-save-and-sync-tooltip = Сохранить план и автоматически синхронизировать его с Zap Drive
ai-document-show-in-warp-drive = Показать в Zap Drive
ai-document-save-as-markdown-file = Сохранить как файл Markdown
ai-document-attach-to-active-session = Прикрепить к активной сессии
ai-document-copy-plan-id = Копировать ID плана
ai-document-plan-id-copied = ID плана скопирован в буфер обмена
ai-conversation-view-in-oz = Показать запуск
ai-conversation-view-in-oz-tooltip = Показать этот запуск агента
ai-block-open-in-github = Открыть в GitHub
ai-block-open-in-code-review = Открыть в ревью кода
ai-block-manage-rules = Управлять правилами
ai-block-review-changes = Просмотреть изменения
ai-block-open-all-in-code-review = Открыть все в ревью кода
ai-block-dont-show-again = Больше не показывать
ai-block-rewind = Откатить
ai-block-rewind-tooltip = Откатить к состоянию до этого блока
ai-block-remove-queued-prompt = Удалить промпт из очереди
ai-block-send-now = Отправить сейчас
ai-block-check-now =  · Проверить сейчас
ai-block-check-now-tooltip = Попросить агента проверить эту команду сейчас, минуя таймер.
ai-block-resume-conversation = Возобновить диалог
ai-block-continue-conversation = Продолжить диалог
ai-block-fork-conversation = Ответвить диалог
ai-block-show-credit-usage-details = Показать сведения об использовании кредитов
ai-block-follow-up-existing-conversation = Продолжить в существующем диалоге
ai-block-accept = Принять
ai-block-auto-approve = Автоодобрение
ai-rule-add-rule = Добавить правило
ai-rule-edit-rule = Редактировать правило
ai-rule-delete-rule = Удалить правило
ai-aws-refresh-credentials = Обновить учетные данные AWS
ai-footer-enable-notifications = Включить уведомления
ai-footer-enable-notifications-tooltip = Установите плагин Warp, чтобы включить расширенные уведомления агента в Zap
ai-footer-notifications-setup-instructions = Инструкции по настройке уведомлений
ai-footer-install-plugin-instructions-tooltip = Посмотреть инструкции по установке плагина Warp
ai-footer-update-warp-plugin = Обновить плагин Warp
ai-footer-plugin-update-available-tooltip = Доступна новая версия плагина Warp
ai-footer-plugin-update-instructions = Инструкции по обновлению плагина
ai-footer-plugin-update-instructions-tooltip = Посмотреть инструкции по обновлению плагина Warp
ai-footer-context-window-usage-tooltip = Использование контекстного окна
ai-footer-choose-environment-tooltip = Выбрать окружение
ai-footer-reasoning-depth-tooltip = Глубина рассуждений
ai-footer-file-explorer = Обозреватель файлов
ai-footer-open-file-explorer = Открыть обозреватель файлов
ai-footer-rich-input = Расширенный ввод
ai-footer-open-rich-input = Открыть расширенный ввод
ai-footer-open-coding-agent-settings = Открыть настройки coding-агента
ai-ask-user-question-placeholder = Введите ответ и нажмите Enter
ai-ask-user-questions-skipped = Вопросы пропущены
ai-ask-user-answered-question = Отвечено на вопрос
ai-ask-user-answered-all-questions = Отвечено на все { $total } вопросов
ai-ask-user-answered-count = Отвечено на { $answered_count } из { $total } вопросов
ai-code-diff-requested-edit-title = Запрошенное изменение
ai-cloud-setup-visit-oz = Открыть настройку агента
ai-inline-code-diff-review-changes = Просмотреть изменения
ai-execution-profile-name-placeholder = напр. «YOLO code»
ai-execution-profile-delete-profile = Удалить профиль
ai-notifications-mark-all-as-read = Отметить все как прочитанные
ai-assistant-copy-transcript-tooltip = Скопировать запись диалога в буфер обмена
code-comment = Комментировать
code-copy-file-path = Копировать путь к файлу
code-select-all = Выделить все
code-replace-all = Заменить все
code-goto-line-placeholder = Номер строки:Столбец
code-open-file-unavailable-remote-tooltip = Открытие файлов недоступно для удаленных сессий
code-view-markdown-preview = Просмотреть предпросмотр Markdown
markdown-display-mode-rendered = Отрендеренный
markdown-display-mode-raw = Исходный
code-review-commit-and-create-pr = Закоммитить и создать PR
notebook-link-text-placeholder = Текст
notebook-link-url-placeholder = Ссылка (веб или файл)
notebook-block-embed = Встроить
notebook-block-divider = Разделитель
notebook-insert-block-tooltip = Вставить блок
notebook-refresh-notebook = Обновить блокнот
notebook-refresh-file = Обновить файл
notebook-open-in-editor = Открыть в редакторе
notebook-sign-in-to-edit = Войдите, чтобы редактировать
editor-custom-keybinding = Другое…
editor-change-keybinding = Изменить сочетание клавиш
autosuggestion-ignore-this-suggestion = Игнорировать эту подсказку
codex-use-latest-model = Использовать последнюю модель codex
zap-launch-visit-repo = Перейти в репозиторий
zap-launch-title = Zap теперь с открытым исходным кодом
zap-launch-description = Вы — наше сообщество — можете участвовать в разработке Zap, используя агент-ориентированный workflow.
zap-launch-contribute-title = Внести вклад
zap-launch-contribute-description = Клиентский код Zap теперь открыт. Начните с навыка /feedback, чтобы открыть issue, и следуйте рекомендациям по внесению вклада здесь.
zap-launch-contribute-link-text = здесь
zap-launch-oad-title = Open Automated Development
zap-launch-oad-description = Репозиторием Zap управляет локальный агент-ориентированный workflow на базе Oz.
zap-launch-auto-model-title = Представляем «auto (open-weights)»
zap-launch-auto-model-description = Мы добавили новую модель auto, которая выбирает лучшую модель с открытыми весами для задачи, например Kimi или MiniMax.
hoa-see-whats-new = Посмотреть, что нового
hoa-finish = Завершить
session-config-get-warping = Начать работу в Zap
uri-custom-uri-invalid = Пользовательский URI недействителен.
context-node-install-nvm = Установить nvm
context-node-install-node = nvm install node
context-node-installed = Установлено
context-chip-change-git-branch = Изменить ветку git
context-chip-view-pull-request = Просмотреть pull request
context-chip-change-working-directory = Изменить рабочий каталог
context-chip-working-directory = Рабочий каталог
settings-ai-repo-placeholder = напр. ~/code-repos/repo
settings-ai-commands-comma-separated-placeholder = Команды, разделенные запятыми
settings-ai-regex-example-placeholder = напр. ls .*
settings-ai-command-supports-regex-placeholder = команда (поддерживает regex)
settings-ai-aws-login-placeholder = aws login
settings-ai-default-placeholder = default
settings-working-directory-path-placeholder = Путь к каталогу
settings-startup-shell-executable-path-placeholder = Путь к исполняемому файлу
settings-agent-providers-base-url-placeholder = https://api.deepseek.com/v1
drive-sharing-only-people-invited = Только приглашенные
drive-sharing-anyone-with-link = Все, у кого есть ссылка
drive-sharing-only-invited-teammates = Только локальный доступ
drive-sharing-teammates-with-link = Локальный доступ по ссылке
terminal-warpify-subshell = Warpify subshell
terminal-warpify-subshell-tooltip = Включить интеграцию shell Zap в этой сессии
terminal-use-agent = Использовать агента
terminal-use-agent-tooltip = Попросить агента Zap о помощи
terminal-give-control-back-to-agent = Вернуть управление агенту
terminal-resume-agent-tooltip = Попросить агента Zap возобновить работу
terminal-voice-input-tooltip = Голосовой ввод
terminal-attach-file-tooltip = Прикрепить файл
terminal-slash-commands-tooltip = Slash-команды
terminal-manage-api-keys-tooltip = Управление API-ключами
terminal-profiles = Профили
terminal-manage-profiles = Управление профилями
terminal-continue-locally = Продолжить локально
terminal-fork-conversation-locally-tooltip = Ответвить этот диалог локально
terminal-open-in-warp = Открыть в Zap
terminal-open-conversation-in-warp-tooltip = Открыть этот диалог в десктопном приложении Zap
terminal-stop-sharing = Прекратить совместный доступ
terminal-copy-session-sharing-link = Копировать ссылку для совместного доступа к сессии
terminal-shared-session-make-editor = Сделать редактором
terminal-shared-session-make-viewer = Сделать наблюдателем
terminal-shared-session-change-role = Изменить роль
terminal-choose-execution-profile-tooltip = Выбрать профиль выполнения AI
terminal-choose-agent-model-tooltip = Выбрать модель агента
terminal-input-cli-agent-rich-input-hint = Скажите агенту, что создать…
terminal-input-enter-prompt-for-agent = Введите промпт для { $agent }…
terminal-input-cloud-agent-hint = Запустите агента
terminal-input-a11y-label = Ввод команды.
terminal-input-a11y-helper = Введите команду shell и нажмите Enter для выполнения. Нажмите cmd-up, чтобы перейти к выводу ранее выполненных команд. Нажмите cmd-l, чтобы снова сфокусироваться на вводе команды.
terminal-input-ai-command-search-hint = Введите «#», чтобы получить предложения AI-команд
terminal-input-run-commands-hint = Выполнить команды
terminal-input-agent-hint-deploy-react-vercel = Zap что угодно, например: Разверните мое React-приложение в Vercel и настройте переменные окружения
terminal-input-agent-hint-debug-python-ci = Zap что угодно, например: Помогите мне отладить, почему мои тесты на Python падают в CI
terminal-input-agent-hint-setup-microservice = Zap что угодно, например: Настройте новый микросервис с Docker и создайте конвейер развертывания
terminal-input-agent-hint-fix-node-memory-leak = Zap что угодно, например: Найдите и устраните утечку памяти в моем приложении на Node.js
terminal-input-agent-hint-backup-postgres = Zap что угодно, например: Создайте скрипт резервного копирования моей базы данных PostgreSQL и настройте его расписание
terminal-input-agent-hint-migrate-mysql-postgres = Zap что угодно, например: Помогите мне перенести данные из MySQL в PostgreSQL
terminal-input-agent-hint-monitor-aws = Zap что угодно, например: Настройте мониторинг и оповещения для моей инфраструктуры AWS
terminal-input-agent-hint-build-fastapi = Zap что угодно, например: Создайте REST API для моего мобильного приложения с помощью FastAPI
terminal-input-agent-hint-optimize-sql = Zap что угодно, например: Помогите оптимизировать мои SQL-запросы, которые выполняются медленно
terminal-input-agent-hint-github-actions = Zap что угодно, например: Создайте workflow GitHub Actions для автоматического развертывания при слиянии
terminal-input-agent-hint-redis-cache = Zap что угодно, например: Настройте кэширование Redis для моего веб-приложения
terminal-input-agent-hint-kubernetes-pods = Zap что угодно, например: Помогите выяснить, почему поды Kubernetes постоянно падают
terminal-input-agent-hint-bigquery-pipeline = Zap что угодно, например: Создайте конвейер данных для обработки CSV-файлов и загрузки их в BigQuery
terminal-input-agent-hint-ssl-https = Zap что угодно, например: Установите SSL-сертификаты и настройте HTTPS для моего домена
terminal-input-agent-hint-refactor-legacy-code = Zap что угодно, например: Помогите мне переработать этот устаревший код, чтобы он использовал современные паттерны проектирования
terminal-input-agent-hint-unit-tests = Zap что угодно, например: Создайте модульные тесты для моего сервиса аутентификации
terminal-input-agent-hint-elk-logs = Zap что угодно, например: Настройте агрегацию журналов с помощью стека ELK для моей распределенной системы
terminal-input-agent-hint-oauth-express = Zap что угодно, например: Помогите мне реализовать аутентификацию OAuth2 в моем приложении на Express.js
terminal-input-agent-hint-optimize-docker = Zap что угодно, например: Оптимизируйте мои образы Docker, чтобы сократить время сборки и размер
terminal-input-agent-hint-ab-testing = Zap что угодно, например: Настройте инфраструктуру A/B-тестирования для моего веб-приложения
terminal-input-steer-agent-hint = Направляйте работающего агента
terminal-input-steer-agent-backspace-hint = Направляйте работающего агента или нажмите backspace, чтобы выйти
terminal-input-follow-up-hint = Задайте уточняющий вопрос
terminal-input-follow-up-backspace-hint = Задайте уточняющий вопрос или нажмите backspace, чтобы выйти
terminal-input-search-queries = Поиск запросов
terminal-input-search-queries-rewind = Поиск запросов для возврата
terminal-input-search-conversations = Поиск диалогов
terminal-input-search-skills = Поиск навыков
terminal-input-search-models = Поиск моделей
terminal-input-search-profiles = Поиск профилей
terminal-input-search-commands = Поиск команд
terminal-input-search-prompts = Поиск промптов
terminal-input-search-indexed-repos = Поиск по проиндексированным репозиториям
terminal-input-search-plans = Поиск планов
terminal-input-choose-agent-model = Выбрать модель агента
terminal-message-new-agent-conversation = {" "}новый диалог /agent
terminal-message-agent-for-new-conversation = /agent для нового диалога
terminal-message-selected-text-attached = выделенный текст прикреплен как контекст
terminal-message-to-remove = {" "}чтобы удалить
terminal-message-to-dismiss = {" "}чтобы скрыть
terminal-message-plan-with-agent = {" "}спланировать с агентом
terminal-message-continue-conversation = {" "}чтобы продолжить диалог
terminal-message-to-execute = {" "}чтобы выполнить
terminal-message-to-send = {" "}чтобы отправить
terminal-message-open-conversation-title = {" "}чтобы открыть «{ $title }»
terminal-message-autodetected = {" "}(определено автоматически){" "}
terminal-message-to-override = {" "}чтобы переопределить
terminal-message-to-navigate = {" "}чтобы перемещаться
terminal-message-to-cycle-tabs = {" "}чтобы переключать вкладки
terminal-message-to-select = {" "}чтобы выбрать
terminal-message-select-save-profile = {" "}выберите и сохраните в профиль
terminal-message-open-plan = {" "}открыть план
terminal-starting-shell = Запуск shell…
terminal-input-no-skills-found = Навыки не найдены
terminal-model-specs-title = Характеристики моделей
terminal-model-specs-description = Бенчмарки Zap, показывающие, насколько хорошо модель работает в нашем harness, как быстро расходует кредиты и как быстро выполняет задачи.
terminal-model-specs-reasoning-level-title = Уровень рассуждений
terminal-model-specs-reasoning-level-description = Повышенные уровни рассуждений расходуют больше кредитов и имеют более высокую задержку, но дают более высокую производительность на сложных задачах.
terminal-model-auto-mode-title = Автоматический режим
terminal-model-auto-mode-description = Auto выбирает лучшую модель для задачи. Cost-efficiency оптимизирует по стоимости, Responsiveness — по скорости ответа.
terminal-model-banner-base-agent = Вы используете базового агента. Модели для полного использования терминала применимы только к агенту полного использования терминала.
terminal-model-banner-full-terminal-agent = Вы используете агента полного использования терминала. Базовые модели применимы только к базовому агенту.
terminal-filter-block-output-placeholder = Фильтровать вывод блока

# =============================================================================
# SECTION: object-surfaces (Owner: agent-i18n-remaining)
# Files: app/src/code_review/**, app/src/notebooks/**, app/src/workflows/**, app/src/drive/**
# =============================================================================

code-review-tooltip-show-file-navigation = Показать навигацию по файлам
code-review-discard-changes = Отменить изменения
code-review-create-pr = Создать PR
code-review-add-diff-set-context = Добавить набор диффов как контекст
code-review-show-saved-comment = Показать сохраненный комментарий
code-review-add-comment = Добавить комментарий
code-review-discard-all = Отменить все
code-review-initialize-codebase = Инициализировать кодовую базу
code-review-initialize-codebase-tooltip = Включает индексацию кодовой базы и WARP.md
code-review-open-repository = Открыть репозиторий
code-review-open-repository-tooltip = Перейдите в репозиторий и инициализируйте его для работы с кодом
code-review-open-file = Открыть файл
code-review-add-file-diff-context = Добавить дифф файла как контекст
code-review-copy-file-path = Копировать путь к файлу
code-review-no-open-changes = Нет открытых изменений
code-review-header-reviewing-changes = Просмотр изменений кода
code-review-search-diff-placeholder = Поиск наборов диффов или веток для сравнения…
code-review-one-comment = 1 комментарий
code-review-copy-text = Копировать текст
code-review-file-level-comment-cannot-edit = Комментарии уровня файла пока нельзя редактировать.
code-review-outdated-comment-cannot-edit = Устаревшие комментарии нельзя редактировать.
code-review-view-in-github = Просмотреть на GitHub
notebook-menu-attach-active-session = Прикрепить к активной сессии
object-menu-open-on-desktop = Открыть в десктопном приложении
notebook-tooltip-restore-from-trash = Восстановить блокнот из корзины
notebook-tooltip-copy-to-personal = Скопировать содержимое блокнота в ваше личное рабочее пространство
notebook-copy-to-personal = Скопировать в личное
notebook-tooltip-copy-to-clipboard = Скопировать содержимое блокнота в буфер обмена
notebook-copy-all = Скопировать все
object-toast-link-copied = Ссылка скопирована в буфер обмена
drive-toast-finished-exporting = Экспорт объектов завершен

# =============================================================================
# SECTION: remaining-settings-tabs-env (Owner: agent-i18n-remaining)
# Files: app/src/settings_view/**, app/src/tab_configs/**, app/src/env_vars/**
# =============================================================================

settings-environment-delete-button = Удалить окружение
settings-language-system-default = Системный по умолчанию
settings-language-english = English
tab-config-open-tab = Открыть вкладку
tab-config-make-default = Сделать по умолчанию
tab-config-already-default = Уже по умолчанию
tab-config-edit-config = Редактировать конфигурацию
env-vars-restore-tooltip = Восстановить переменные окружения из корзины
env-vars-variables-label = Переменные

# =============================================================================
# SECTION: onboarding-callout (Owner: agent-i18n-remaining)
# Files: crates/onboarding/src/callout/view.rs
# =============================================================================

onboarding-callout-meet-input-title = Знакомьтесь с вводом Zap
onboarding-callout-meet-input-text-prefix = Ввод в терминале принимает как команды терминала, так и промпты агента и автоматически определяет, что вы вводите. Используйте
onboarding-callout-meet-input-text-suffix = чтобы зафиксировать ввод в режиме агента (естественный язык) или в режиме терминала (команды).
onboarding-callout-talk-agent-title = Поговорите с агентом
onboarding-callout-talk-agent-text = Вы можете общаться с агентом на естественном языке. Отправьте запрос ниже, чтобы начать: Какие тесты есть в этом репозитории, как они устроены и что они покрывают?
onboarding-callout-skip = Пропустить
onboarding-callout-submit = Отправить
onboarding-callout-finish = Завершить
onboarding-callout-meet-terminal-title = Знакомьтесь с вашим вводом в терминале
onboarding-callout-meet-updated-terminal-title = Знакомьтесь с обновленным вводом в терминале
onboarding-callout-meet-terminal-text-prefix = Выполняйте команды в терминале или используйте
onboarding-callout-meet-terminal-text-suffix = чтобы запустить агента или отправить ему текст.
onboarding-callout-nl-overrides-title = Переопределение естественным языком
onboarding-callout-nl-overrides-text-prefix = Вы всегда можете переопределить любое автоматическое определение, используя
onboarding-callout-nl-support-title = Поддержка естественного языка
onboarding-callout-nl-support-text-prefix = Ввод на естественном языке по умолчанию выключен. Если включить его, вы сможете писать запросы обычным языком, и Zap будет автоматически определять запросы для агента. Переопределить их можно в любой момент с помощью
onboarding-callout-enable-nl-detection = Включить определение естественного языка
onboarding-callout-new-agent-title = Представляем новый опыт работы с агентом в Zap
onboarding-callout-new-agent-text = Диалоги с агентом теперь открываются в отдельном представлении за пределами терминала. Чтобы в любой момент вернуться в терминал, просто нажмите ESC.
onboarding-callout-updated-agent-input-title = Обновленный ввод для агента
onboarding-callout-updated-agent-input-project-text = Ввод для агента теперь по умолчанию распознает как естественный язык, так и команды. Используйте !, чтобы зафиксировать ввод в режиме bash и писать команды.\n\nОтправьте запрос ниже, чтобы агент инициализировал этот проект, или нажмите ⊗, чтобы очистить ввод и написать свой!
onboarding-callout-skip-initialization = Пропустить инициализацию
onboarding-callout-initialize = Инициализировать
onboarding-callout-updated-agent-input-text = Ввод для агента теперь по умолчанию распознает как естественный язык, так и команды. Используйте !, чтобы зафиксировать ввод в режиме bash и писать команды.
onboarding-callout-back-terminal = Вернуться в терминал

# =============================================================================
# SECTION: language (Owner: foundation)
# Files: app/src/settings_view/appearance_page.rs (Language widget + restart modal)
# =============================================================================

language-widget-label = Язык
language-widget-secondary = Перезапустите Zap, чтобы изменение вступило в силу полностью.
language-restart-required-title = Язык изменен
language-restart-required-body = Язык интерфейса Zap обновлен. Часть текста сменится сразу, но для применения изменения везде требуется полный перезапуск.

# =============================================================================
# SECTION: settings (Owner: agent-settings)
# Files: app/src/settings_view/**
# =============================================================================

# --- ANCHOR-SUB-MOD-NAV (agent-settings-mod) ---
# settings_view/mod.rs SettingsSection Display labels + context menu pane actions

# Sidebar / SettingsSection labels (Display impl)
settings-section-about = О программе
# Zap: settings-section-account removed alongside the Account settings page.
settings-section-mcp-servers = Серверы MCP
settings-section-billing-and-usage = Оплата и использование
settings-section-appearance = Внешний вид
settings-section-features = Возможности
settings-section-keybindings = Сочетания клавиш
settings-section-referrals = Реферальная программа
settings-section-shared-blocks = Общие блоки
settings-section-warp-drive = Zap Drive
settings-section-warpify = Warpify
settings-section-network = Сеть
settings-section-cloud-sync = Облачная синхронизация
settings-section-ai = AI
settings-section-warp-agent = Агент Zap
settings-section-agent-profiles = Профили
settings-section-agent-mcp-servers = Серверы MCP
settings-section-agent-providers = Провайдеры
settings-section-knowledge = Знания
settings-section-third-party-cli-agents = Сторонние CLI-агенты
settings-section-code = Код
settings-section-editor-and-code-review = Редактор и проверка кода
settings-section-cloud-environments = Окружения
settings-section-oz-cloud-api-keys = API-ключи агента
settings-title = Настройки

# Context menu items (split / close pane)
settings-pane-split-right = Разделить панель вправо
settings-pane-split-left = Разделить панель влево
settings-pane-split-down = Разделить панель вниз
settings-pane-split-up = Разделить панель вверх
settings-pane-close = Закрыть панель

# Debug toggle setting descriptions (command palette)
settings-debug-show-init-block = Показать блок инициализации
settings-debug-hide-init-block = Скрыть блок инициализации
settings-debug-show-inband-blocks = Показать встроенные блоки команд
settings-debug-hide-inband-blocks = Скрыть встроенные блоки команд

# --- ANCHOR-SUB-ABOUT (agent-settings-about) ---
# 此锚点下放 settings_view/about_page.rs + main_page.rs 字符串
# 命名前缀:settings-about-* / settings-main-*

# about_page.rs
settings-about-copyright = Copyright 2026 Zap
settings-about-automatic-updates-label = Автоматические обновления
settings-about-automatic-updates-description = Если включено, Zap проверяет наличие новых версий в фоне и загружает установщик в локальный кеш. Запущенный Zap не изменяется, пока вы сами не нажмете «Установить сейчас», чтобы запустить установщик.
settings-about-update-checking = Проверка обновлений…
settings-about-update-up-to-date = Zap обновлен.
settings-about-update-available = Доступна новая версия { $version }.
settings-about-update-downloading = Загрузка { $version }… { $progress }
settings-about-update-downloading-init = Загрузка { $version }…
settings-about-update-ready = { $version } загружена и готова к установке.
settings-about-update-check-now = Проверить обновления
settings-about-update-open-release = Скачать с GitHub
settings-about-update-install-now = Установить сейчас
settings-about-update-install-hint-macos = Откроется установщик — перетащите Zap в папку «Программы», чтобы завершить.
settings-about-update-install-hint-windows = Запустится мастер установки — следуйте подсказкам, чтобы завершить обновление.
settings-about-update-install-hint-linux = Файл AppImage будет заменен на месте, и Zap перезапустится.
settings-about-export-logs = Экспортировать журналы…
settings-about-export-logs-description = Собирает недавние журналы приложения (а также журналы MCP и обновлений, если они есть) и диагностическую сводку в zip-архив, для которого вы выбираете место сохранения, чтобы можно было поделиться им для устранения неполадок.
settings-about-export-logs-success = Журналы экспортированы в { $path }
settings-about-export-logs-failure = Не удалось экспортировать журналы: { $error }

# Zap: main_page.rs (Account / version / autoupdate) strings removed alongside
# the Account settings page. The About page now owns version / update CTAs.


# --- ANCHOR-SUB-MCP (agent-settings-mcp) ---
# 此锚点下放 settings_view/mcp_servers_page.rs 字符串
# 命名前缀:settings-mcp-*
settings-mcp-page-title = Серверы MCP
settings-mcp-logout-success-named = Выполнен выход из сервера MCP {$name}
settings-mcp-logout-success = Выполнен выход из сервера MCP
settings-mcp-install-modal-busy = Завершите текущую установку MCP, прежде чем открывать другую ссылку установки.
settings-mcp-unknown-server = Неизвестный сервер MCP «{$name}»
settings-mcp-install-from-link-failed = Сервер MCP «{$name}» нельзя установить по этой ссылке.

# ---- destructive_mcp_confirmation_dialog.rs ----
settings-mcp-confirm-delete-local-title = Удалить сервер MCP?
settings-mcp-confirm-delete-local-description = Сервер MCP будет деинсталлирован и удален с этого устройства.
settings-mcp-confirm-delete-shared-title = Удалить сервер MCP?
settings-mcp-confirm-delete-shared-description = Это удалит сохраненный сервер MCP с этого устройства.
settings-mcp-confirm-unshare-title = Удалить сохраненный сервер MCP?
settings-mcp-confirm-unshare-description = Это удалит сохраненный сервер MCP с этого устройства.
settings-mcp-confirm-delete-button = Удалить MCP
settings-mcp-confirm-remove-from-team-button = Удалить сохраненную копию
settings-mcp-confirm-cancel-button = Отмена

# ---- edit_page.rs ----
settings-mcp-edit-save = Сохранить
settings-mcp-edit-edit-variables = Изменить переменные
settings-mcp-edit-delete = Удалить MCP
settings-mcp-edit-remove-from-team = Удалить сохраненную копию
settings-mcp-edit-editing-disabled-banner = Этот сервер MCP нельзя редактировать в этом представлении.
settings-mcp-edit-add-new-title = Добавить новый сервер MCP
settings-mcp-edit-edit-named-title = Изменить сервер MCP { $name }
settings-mcp-edit-edit-title = Изменить сервер MCP
settings-mcp-edit-logout-tooltip = Выйти
settings-mcp-edit-secrets-error = Этот сервер MCP содержит секреты. Откройте Настройки > Конфиденциальность, чтобы изменить настройки скрытия секретов.
settings-mcp-edit-no-server-error = Сервер MCP не указан.
settings-mcp-edit-multiple-servers-error = Нельзя добавить несколько серверов MCP при редактировании одного сервера.

# ---- installation_modal.rs ----
settings-mcp-install-modal-title = Установить { $name }
settings-mcp-install-modal-source-shared = Сохраненный пресет
settings-mcp-install-modal-source-other-device = С другого устройства
settings-mcp-install-modal-cancel = Отмена
settings-mcp-install-modal-install = Установить
settings-mcp-install-modal-no-server = Сервер MCP не выбран

# ---- list_page.rs ----
settings-mcp-list-description = Добавляйте серверы MCP, чтобы расширить возможности Агента Zap. Серверы MCP предоставляют агентам источники данных или инструменты через стандартизированный интерфейс, по сути работая как плагины. Добавьте свой сервер или используйте пресеты, чтобы начать с популярных серверов.
settings-mcp-list-learn-more = Подробнее.
settings-mcp-list-empty-state = Как только вы добавите сервер MCP, он появится здесь.
settings-mcp-list-no-search-results = Ничего не найдено
settings-mcp-list-search-placeholder = Поиск серверов MCP
settings-mcp-list-add-button = Добавить
settings-mcp-list-file-based-toggle-label = Автозапуск серверов из сторонних агентов
settings-mcp-list-file-based-description = Автоматически обнаруживать и запускать серверы MCP из глобальных конфигурационных файлов сторонних AI-агентов (например, в вашей домашней папке). Серверы, обнаруженные внутри репозитория, никогда не запускаются автоматически — их нужно включать по отдельности в разделах «Обнаружено из» ниже.
settings-mcp-list-file-based-supported-providers = См. поддерживаемые провайдеры.
settings-mcp-list-template-available-to-install = Доступно для установки
settings-mcp-list-file-based-detected = Обнаружено из файла конфигурации
settings-mcp-list-toast-server-updated = Сервер MCP обновлен
settings-mcp-list-section-my-mcps = Мои серверы MCP
settings-mcp-list-section-shared-by-warp-and-team = Доступно от Zap и { $name }
settings-mcp-list-section-shared-by-warp-and-other-devices = Общий доступ от Zap и с других устройств
settings-mcp-list-section-shared-from-warp = Общий доступ от Zap
settings-mcp-list-section-detected-from = Обнаружено из { $provider }
settings-mcp-list-chip-global = глобальный
settings-mcp-list-chip-shared-by-creator = Общий доступ: { $creator }
settings-mcp-list-chip-shared-by-team-member = Сохраненный пресет
settings-mcp-list-chip-from-another-device = С другого устройства

# ---- server_card.rs ----
settings-mcp-card-tooltip-show-logs = Показать журналы
settings-mcp-card-tooltip-log-out = Выйти
settings-mcp-card-tooltip-share-server = Поделиться сервером
settings-mcp-card-tooltip-edit = Изменить
settings-mcp-card-tooltip-update-available = Доступно обновление сервера
settings-mcp-card-button-view-logs = Открыть журналы
settings-mcp-card-button-edit-config = Изменить конфигурацию
settings-mcp-card-button-set-up = Настроить
settings-mcp-card-tools-none = Нет доступных инструментов
settings-mcp-card-tools-available = Доступно инструментов: { $count }
settings-mcp-card-status-offline = Не в сети
settings-mcp-card-status-starting = Запуск сервера…
settings-mcp-card-status-authenticating = Аутентификация…
settings-mcp-card-status-shutting-down = Остановка…

# ---- update_modal.rs ----
settings-mcp-update-modal-default-name = Сервер
settings-mcp-update-modal-title = Обновить { $name }
settings-mcp-update-modal-description = Для этого сервера доступно обновлений: { $count }. Какое применить?
settings-mcp-update-modal-publisher-another-device = другое устройство
settings-mcp-update-modal-publisher-team-member = локальный источник
settings-mcp-update-modal-update-from = Обновить с { $publisher }
settings-mcp-update-modal-version = Версия { $version }
settings-mcp-update-modal-cancel = Отмена
settings-mcp-update-modal-update = Обновить
settings-mcp-update-modal-no-updates = Нет доступных обновлений

# --- ANCHOR-SUB-PLATFORM (agent-settings-platform) ---
# 此锚点下放 settings_view/platform_page.rs 字符串
# 命名前缀:settings-platform-*
settings-platform-section-title = API-ключи агента
settings-platform-description = Создавайте ключи API и управляйте ими, чтобы разрешить локальным агентам доступ к вашему аккаунту Zap.
    Для получения дополнительной информации посетите
settings-platform-documentation-link = документацию.
settings-platform-create-button = + Создать ключ API
settings-platform-modal-title-new = Новый ключ API
settings-platform-modal-title-save = Сохраните ваш ключ
settings-platform-toast-deleted = Ключ API удален
settings-platform-column-name = Название
settings-platform-column-key = Ключ
settings-platform-column-scope = Область действия
settings-platform-column-created = Создан
settings-platform-column-last-used = Последнее использование
settings-platform-column-expires-at = Истекает
settings-platform-value-never = Никогда
settings-platform-scope-personal = Личный
settings-platform-scope-team = Командный
settings-platform-zero-state-title = Нет ключей API
settings-platform-zero-state-description = Создайте ключ, чтобы управлять внешним доступом к Zap
settings-platform-create-api-key-description-personal = Этот ключ API привязан к вашему пользователю и может выполнять запросы к вашему аккаунту Zap.
settings-platform-create-api-key-description-team = Этот ключ API привязан к вашей команде и может выполнять запросы от имени вашей команды.
settings-platform-create-api-key-name-placeholder = Ключ API Zap
settings-platform-create-api-key-expiration-one-day = 1 день
settings-platform-create-api-key-expiration-thirty-days = 30 дней
settings-platform-create-api-key-expiration-ninety-days = 90 дней
settings-platform-create-api-key-label-type = Тип
settings-platform-create-api-key-label-expiration = Срок действия
settings-platform-create-api-key-error-no-current-team = Невозможно создать командный ключ API, так как текущая команда отсутствует.
settings-platform-create-api-key-error-create-failed = Не удалось создать ключ API. Попробуйте еще раз.
settings-platform-create-api-key-secret-once = Этот секретный ключ показывается только один раз. Скопируйте и сохраните его в надежном месте.
settings-platform-create-api-key-copied = Скопировано
settings-platform-create-api-key-done = Готово
settings-platform-create-api-key-creating = Создание…
settings-platform-create-api-key-create = Создать ключ
settings-platform-create-api-key-toast-secret-copied = Секретный ключ скопирован.

# --- ANCHOR-SUB-KEYBINDINGS (agent-settings-keybindings) ---
settings-keybindings-search-placeholder = Поиск по названию или по клавишам (например, «cmd d»)
settings-keybindings-conflict-warning = Это сочетание клавиш конфликтует с другими сочетаниями клавиш
settings-keybindings-button-default = По умолчанию
settings-keybindings-button-cancel = Отмена
settings-keybindings-button-clear = Очистить
settings-keybindings-button-save = Сохранить
settings-keybindings-press-new-shortcut = Нажмите новое сочетание клавиш
settings-keybindings-description = Добавьте собственные сочетания клавиш для перечисленных ниже действий.
settings-keybindings-use-prefix = Используйте
settings-keybindings-use-suffix = , чтобы открыть эти сочетания клавиш в боковой панели в любой момент.
settings-keybindings-not-synced-tooltip = Сочетания клавиш хранятся локально на этом компьютере
settings-keybindings-subheader = Настройка сочетаний клавиш
settings-keybindings-command-column = Команда

# --- ANCHOR-SUB-REFERRALS (agent-settings-referrals) ---
settings-referrals-page-title = Пригласите друга в Zap
settings-referrals-anonymous-header = Реферальная программа недоступна в локальных сборках Zap
settings-referrals-sign-up = Недоступно локально
settings-referrals-link-label = Ссылка
settings-referrals-email-label = Email
settings-referrals-link-error = Не удалось загрузить реферальный код.
settings-referrals-loading = Загрузка…
settings-referrals-copy-link-button = Копировать ссылку
settings-referrals-email-send-button = Отправить
settings-referrals-email-sending-button = Отправка…
settings-referrals-link-copied-toast = Ссылка скопирована.
settings-referrals-email-success-toast = Письма успешно отправлены.
settings-referrals-email-failure-toast = Не удалось отправить письма. Попробуйте еще раз.
settings-referrals-email-empty-error = Пожалуйста, введите email.
settings-referrals-email-invalid-error = Пожалуйста, убедитесь, что этот адрес email корректен: { $email }
settings-referrals-reward-intro = Получайте эксклюзивные подарки Zap за приглашенных друзей*
settings-referrals-claimed-count-singular = Текущий реферал
settings-referrals-claimed-count-plural = Текущие рефералы
settings-referrals-terms-link = Действуют некоторые ограничения.
settings-referrals-terms-contact = { " " }Если у вас есть вопросы о реферальной программе, свяжитесь с нами: referrals@warp.dev.
settings-referrals-reward-theme = Эксклюзивная тема
settings-referrals-reward-keycaps = Клавиши + стикеры
settings-referrals-reward-tshirt = Футболка
settings-referrals-reward-notebook = Блокнот
settings-referrals-reward-cap = Бейсболка
settings-referrals-reward-hoodie = Худи
settings-referrals-reward-hydroflask = Премиальная бутылка Hydro Flask
settings-referrals-reward-backpack = Рюкзак

# --- ANCHOR-SUB-WARPIFY (agent-settings-warpify) ---
settings-warpify-page-title = Warpify
settings-warpify-description-prefix = Настройте, должен ли Zap пытаться применять «Warpify» (добавление поддержки блоков, режимов ввода и т. д.) к определенным оболочкам.
settings-warpify-learn-more = Подробнее
settings-warpify-section-subshells = Вложенные оболочки
settings-warpify-section-subshells-subtitle = Поддерживаемые вложенные оболочки: bash, zsh и fish.
settings-warpify-section-ssh = SSH
settings-warpify-section-ssh-subtitle = Применяйте Warpify к интерактивным SSH-сессиям.
settings-warpify-added-commands = Добавленные команды
settings-warpify-denylisted-commands = Команды в черном списке
settings-warpify-denylisted-hosts = Хосты в черном списке
settings-warpify-command-placeholder = команда (поддерживает регулярные выражения)
settings-warpify-host-placeholder = хост (поддерживает регулярные выражения)
settings-warpify-enable-ssh = Warpify SSH-сессии
settings-warpify-install-ssh-extension = Установить SSH-расширение
settings-warpify-install-ssh-extension-description = Определяет поведение при установке SSH-расширения Zap, если на удаленном хосте оно не установлено.
settings-warpify-use-tmux = Использовать Warpify через Tmux
settings-warpify-tmux-description = Обертка tmux для ssh работает во многих случаях, где стандартная не работает, но для применения Warpify может потребоваться нажать кнопку. Вступает в силу в новых вкладках.
settings-warpify-ssh-tmux-toggle-binding-label = Обнаружение SSH-сессий для Warpify

# --- ANCHOR-SUB-NETWORK (network-settings) ---
# Global HTTP proxy settings page (see Issue #72).
settings-network-page-title = Сеть
settings-network-header = HTTP-прокси
settings-network-description = Настройте глобальный прокси для всех исходящих HTTP / WebSocket-запросов. Нажмите Enter после редактирования поля, чтобы сохранить.\nНовые запросы (список моделей BYOP, проверка соединения, загрузка диалогов и т. д.) применяются сразу; долгоживущие клиенты, создаваемые при запуске (автообновление, список изменений), требуют перезапуска приложения.
settings-network-mode-label = Режим прокси
settings-network-mode-description = Системный следует за ОС / переменными окружения (по умолчанию); Пользовательский использует URL ниже; Выключено отключает прокси полностью.
settings-network-mode-system = Системный
settings-network-mode-custom = Пользовательский
settings-network-mode-off = Отключен
settings-network-url-label = URL прокси
settings-network-url-placeholder = http://proxy.example.com:8080
settings-network-url-description = например, http://proxy.corp:8080
settings-network-username-label = Имя пользователя
settings-network-username-placeholder = Имя пользователя (необязательно)
settings-network-username-description = Если прокси требует Basic Auth, укажите здесь имя пользователя.
settings-network-password-label = Пароль
settings-network-password-placeholder = Пароль (сохраняется в связку ключей ОС при отправке)
settings-network-password-description = Отправленный пароль хранится в связке ключей ОС (а не в settings.toml).
settings-network-no-proxy-label = Список исключений (no-proxy)
settings-network-no-proxy-placeholder = localhost,127.0.0.1,.internal
settings-network-no-proxy-description = Хосты через запятую.
settings-network-save = Сохранить
settings-network-clear = Очистить
settings-network-test-button = Проверить соединение
settings-network-test-idle-tcp = Проверяет host:port прокси по TCP. Тестирует доступность самого прокси, а не выход в интернет — подходит для прокси, работающих только во внутренней сети.
settings-network-test-idle-http = Отправляет GET-запрос к {$url} через текущую конфигурацию. Тестирует выход в интернет.
settings-network-test-running = Проверка…
settings-network-test-success-tcp = ✅ Прокси доступен ({$latency} мс)
settings-network-test-success-http = ✅ Интернет доступен ({$latency} мс)
settings-network-test-failed-tcp = ❌ Прокси недоступен: {$error}
settings-network-test-failed-http = ❌ Не удалось подключиться: {$error}

# --- ANCHOR-SUB-CLOUD-SYNC (agent-settings-cloud-sync) ---
# Cloud Sync settings page
settings-cloud-sync-description = Настройте облачную синхронизацию через GitHub Gist или Gitee Gist. Ваши настройки будут зашифрованы и сохранены в секретный Gist.
settings-cloud-sync-scope-note = Сейчас синхронизируются только конфигурации управляемых SSH-серверов.
settings-cloud-sync-platform-label = Платформа синхронизации
settings-cloud-sync-platform-description = Выберите облачный сервис для синхронизации
settings-cloud-sync-token-label = Токен доступа
settings-cloud-sync-token-description = Персональный токен доступа с областью действия gist
settings-cloud-sync-token-placeholder = Введите токен доступа…
settings-cloud-sync-operations-header = Операции синхронизации
settings-cloud-sync-upload-label = Выгрузить
settings-cloud-sync-download-label = Скачать
settings-cloud-sync-status-header = Статус синхронизации
settings-cloud-sync-local-version-label = Локальная версия
settings-cloud-sync-last-time-label = Время последней синхронизации
settings-cloud-sync-last-platform-label = Платформа последней синхронизации
settings-cloud-sync-local-version = Локальная версия: {$version}
settings-cloud-sync-last-time = Время последней синхронизации: {$time}
settings-cloud-sync-last-platform = Платформа последней синхронизации: {$platform}
settings-cloud-sync-na = Н/Д
settings-cloud-sync-never = Никогда
settings-cloud-sync-syncing-upload = Выгрузка на {$platform}…
settings-cloud-sync-syncing-download = Скачивание с {$platform}…
settings-cloud-sync-success-upload = Успешно выгружено на {$platform} (версия v{$version})
settings-cloud-sync-success-download = Успешно скачано с {$platform} (версия v{$version})
settings-cloud-sync-already-up-to-date = Уже актуальная версия (v{$version}), синхронизация не требуется
settings-cloud-sync-failed = Ошибка: {$error}
settings-cloud-sync-conflict-status = Конфликт: локальная v{$local} против удаленной v{$remote}
settings-cloud-sync-conflict-status-equal = Версии совпадают: локальная v{$local} = удаленная v{$remote}
settings-cloud-sync-token-not-configured = Токен {$platform} не настроен
settings-cloud-sync-conflict-title = Конфликт версий
settings-cloud-sync-conflict-description = Удаленная версия (v{$remote}) новее локальной (v{$local}). Принудительная выгрузка перезапишет удаленные данные.
settings-cloud-sync-conflict-description-equal = Удаленная и локальная версии идентичны. Принудительная выгрузка перезапишет удаленные данные.
settings-cloud-sync-force-upload = Выгрузить принудительно
settings-cloud-sync-download-confirm-title = Подтверждение скачивания
settings-cloud-sync-download-confirm-description = Скачивание заменит все локальные конфигурации SSH-серверов удаленной версией. Это действие нельзя отменить.
settings-cloud-sync-download-confirm-button = Подтвердить скачивание
settings-cloud-sync-upload-confirm-title = Подтверждение выгрузки
settings-cloud-sync-upload-confirm-description = Выгрузка перезапишет все удаленные конфигурации SSH-серверов локальной версией. Gist не хранит историю, поэтому это действие нельзя отменить.
settings-cloud-sync-upload-confirm-button = Подтвердить выгрузку
settings-cloud-sync-clear = Очистить
settings-cloud-sync-validating = Проверка токена…
settings-cloud-sync-token-valid = Токен действителен ({$username})
settings-cloud-sync-token-invalid = Недействительный токен: {$error}
settings-cloud-sync-auto-sync-label = Автосинхронизация
settings-cloud-sync-auto-sync-description = Автоматически выгружать при изменении конфигурации и скачивать при запуске приложения

# --- ANCHOR-SUB-AI-PAGE (agent-settings-ai-page) ---
# Section / sub-headers
settings-ai-warp-agent-header = Агент Zap
settings-ai-active-ai-section = Активный AI
settings-ai-input-section = Ввод
settings-ai-mcp-servers-section = MCP-серверы
settings-ai-knowledge-section = Знания
settings-ai-voice-section = Голос
settings-ai-other-section = Другое
settings-ai-third-party-cli-section = Сторонние CLI-агенты
settings-ai-experimental-section = Экспериментальные
settings-ai-aws-bedrock-section = AWS Bedrock
settings-ai-agents-header = Агенты
settings-ai-profiles-header = Профили
settings-ai-models-subheader = Модели
settings-ai-permissions-subheader = Разрешения
settings-ai-usage-header = Использование
settings-ai-credits-label = Кредиты

# Active AI toggle labels
settings-ai-next-command-label = Следующая команда
settings-ai-prompt-suggestions-label = Подсказки для промптов
settings-ai-suggested-code-banners-label = Баннеры предлагаемого кода
settings-ai-natural-language-autosuggestions-label = Автоподсказки на естественном языке
settings-ai-git-operations-autogen-label = Генерация коммитов и Pull Request

# Permissions dropdown options
settings-ai-permission-agent-decides = Агент решает
settings-ai-permission-always-allow = Всегда разрешать
settings-ai-permission-always-ask = Всегда спрашивать
settings-ai-permission-ask-on-first-write = Спрашивать при первой записи
settings-ai-permission-read-only = Только чтение
settings-ai-permission-supervised = Под контролем
settings-ai-permission-allow-specific-dirs = Разрешать в определенных каталогах

# Permission row labels
settings-ai-apply-code-diffs = Применение диффов кода
settings-ai-read-files = Чтение файлов
settings-ai-execute-commands = Выполнение команд
settings-ai-interact-running-commands = Взаимодействие с выполняющимися командами
settings-ai-call-mcp-servers = Вызов MCP-серверов
settings-ai-command-denylist = Черный список команд
settings-ai-command-denylist-description = Регулярные выражения для команд, перед выполнением которых агент Zap всегда должен запрашивать разрешение.
settings-ai-command-allowlist = Белый список команд
settings-ai-command-allowlist-description = Регулярные выражения для команд, которые агент Zap может выполнять автоматически.
settings-ai-directory-allowlist = Белый список каталогов
settings-ai-directory-allowlist-description = Предоставить агенту доступ к файлам в определенных каталогах.
settings-ai-mcp-allowlist = Белый список MCP
settings-ai-mcp-allowlist-description = Разрешить агенту Zap вызывать эти MCP-серверы.
settings-ai-mcp-denylist = Черный список MCP
settings-ai-mcp-denylist-description = Агент Zap всегда будет запрашивать разрешение перед вызовом любых MCP-серверов из этого списка.
settings-ai-info-banner-managed-by-workspace = Некоторые ваши разрешения управляются рабочим пространством.

# Models / Profiles
settings-ai-base-model = Базовая модель
settings-ai-base-model-description = Эта модель — основной движок агента Zap. Она обрабатывает большинство взаимодействий и при необходимости привлекает другие модели для таких задач, как планирование или генерация кода. Zap может автоматически переключаться на альтернативные модели в зависимости от их доступности или для вспомогательных задач, например суммаризации диалога.
settings-ai-show-model-picker-in-prompt = Показывать выбор модели в промпте
settings-ai-codebase-context = Контекст кодовой базы
settings-ai-codebase-context-description = Разрешить агенту Zap создавать структуру вашей кодовой базы для использования в качестве контекста. Код никогда не сохраняется на наших серверах.
settings-ai-add-profile = Добавить профиль
settings-ai-agents-description = Задайте границы работы агента. Выберите, к чему он имеет доступ, насколько он автономен и когда должен запрашивать ваше одобрение. Также можно тонко настроить поведение — ввод на естественном языке, осведомленность о кодовой базе и многое другое.
settings-ai-profiles-description = Профили позволяют определить, как работает агент: какие действия он может выполнять, когда требуется одобрение и какие модели используются для задач вроде кодинга и планирования. Также их можно ограничить отдельными проектами.

# Anonymous / org gates
settings-ai-sign-up = Включить локальный AI
settings-ai-anonymous-create-account = Локальные функции AI не требуют аккаунта.
settings-ai-org-enforced-tooltip = Этот параметр принудительно задан настройками вашей организации и не может быть изменен.
settings-ai-restricted-billing = Ограничено из-за проблемы с оплатой
settings-ai-unlimited = Без ограничений

# AI Input section
settings-ai-show-input-hint-text = Показывать подсказку в поле ввода
settings-ai-show-agent-tips = Показывать советы агента
settings-ai-show-agent-zero-state-hints = Показывать подсказки сочетаний клавиш агента
settings-ai-include-agent-commands-in-history = Включать команды, выполненные агентом, в историю
settings-ai-autodetect-agent-prompts = Автоматически распознавать промпты агента во вводе терминала
settings-ai-autodetect-terminal-commands = Автоматически распознавать команды терминала во вводе агента
settings-ai-natural-language-detection = Распознавание естественного языка
settings-ai-natural-language-denylist = Черный список естественного языка
settings-ai-natural-language-denylist-description = Команды из этого списка никогда не будут распознаны как естественный язык.
settings-ai-let-us-know = Расскажите нам

# MCP Servers
settings-ai-learn-more = Подробнее
settings-ai-add-server = Добавить сервер
settings-ai-manage-mcp-servers = Управлять MCP-серверами
settings-ai-file-based-mcp-toggle = Автоматически запускать серверы из сторонних агентов
settings-ai-file-based-mcp-supported-providers = См. поддерживаемых провайдеров.
settings-ai-mcp-dropdown-header = Выберите MCP-серверы

# Knowledge / Rules
settings-ai-rules-label = Правила
settings-ai-suggested-rules-label = Предлагаемые правила
settings-ai-suggested-rules-description = Позвольте AI предлагать правила для сохранения на основе ваших взаимодействий.
settings-ai-manage-rules = Управлять правилами
settings-ai-rules-description = Правила помогают агенту Zap следовать вашим соглашениям — как в кодовых базах, так и в отдельных workflow.

# Voice
settings-ai-voice-input-label = Голосовой ввод
settings-ai-voice-key = Клавиша активации голосового ввода
settings-ai-voice-key-hint = Нажмите и удерживайте для активации.

# Other section
settings-ai-show-use-agent-footer = Показывать нижнюю панель «Использовать агента»
settings-ai-use-agent-footer-description = Показывает подсказку использовать агента с включенным «Полным управлением терминалом» в долго выполняющихся командах.
settings-ai-show-conversation-history = Показывать историю диалогов на панели инструментов
settings-ai-thinking-display = Отображение размышлений агента
settings-ai-thinking-display-description = Управляет отображением цепочек рассуждений и размышлений.
settings-ai-conversation-layout-label = Предпочтительная раскладка при открытии существующих диалогов агента
settings-ai-conversation-layout-newtab = Новая вкладка
settings-ai-conversation-layout-splitpane = Разделенная панель
settings-ai-toolbar-layout = Расположение панели инструментов

# Third-party CLI agents
settings-ai-show-coding-agent-toolbar = Показывать панель инструментов coding-агента
settings-ai-auto-show-rich-input = Автоматически показывать/скрывать Rich Input в зависимости от состояния агента
settings-ai-auto-show-rich-input-tooltip = Требуется плагин Warp для вашего coding-агента
settings-ai-auto-open-rich-input = Автоматически открывать Rich Input при старте сессии coding-агента
settings-ai-auto-dismiss-rich-input = Автоматически скрывать Rich Input после отправки промпта
settings-ai-toolbar-commands-label = Команды, включающие панель инструментов
settings-ai-toolbar-commands-description = Добавьте шаблоны регулярных выражений, чтобы показывать панель coding-агента для совпадающих команд.
settings-ai-per-agent-section = Установленные агенты
settings-ai-per-agent-scanning = Идет поиск установленных агентов…
settings-ai-per-agent-empty = Установленные CLI-агенты не найдены.
settings-ai-per-agent-agent-col = Агент
settings-ai-per-agent-toolbar-col = Панель инструментов
settings-ai-per-agent-tab-menu-col = Меню вкладок
settings-ai-per-agent-titlebar-col = Строка заголовка
settings-ai-coding-agent-other = Другое
settings-ai-coding-agent-select-header = Выберите coding-агента

# Experimental / Agent
settings-ai-cloud-agent-computer-use = Computer Use в агентах
settings-ai-cloud-agent-computer-use-description = Включить Computer Use в диалогах агента, запущенных из приложения Zap.

# AWS Bedrock
settings-ai-aws-bedrock-toggle = Использовать учетные данные AWS Bedrock
settings-ai-aws-bedrock-description = Zap загружает и отправляет локальные учетные данные AWS CLI для моделей, поддерживающих Bedrock.
settings-ai-aws-bedrock-description-managed = Zap загружает и отправляет локальные учетные данные AWS CLI для моделей, поддерживающих Bedrock. Этот параметр управляется вашей организацией.
settings-ai-aws-login-command = Команда входа
settings-ai-aws-profile = Профиль AWS
settings-ai-aws-auto-login = Автоматически запускать команду входа
settings-ai-aws-auto-login-description = Если включено, команда входа будет запускаться автоматически при истечении срока учетных данных AWS Bedrock.
settings-ai-refresh = Обновить

# --- ANCHOR-SUB-FEATURES (agent-settings-features) ---
# settings_view/features_page.rs P0 + P1(category + toggle labels)
# 命名前缀:settings-features-*
settings-features-category-general = Общие
settings-features-category-session = Сессия
settings-features-category-keys = Клавиши
settings-features-category-text-editing = Редактирование текста
settings-features-category-terminal-input = Ввод в терминале
settings-features-category-terminal = Терминал
settings-features-category-notifications = Уведомления
settings-features-category-workflows = Workflows
settings-features-category-system = Система
settings-features-open-links-in-desktop = Открывать ссылки в настольном приложении
settings-features-open-links-in-desktop-tooltip = Автоматически открывать ссылки в настольном приложении, когда это возможно.
settings-features-restore-session = Восстанавливать окна, вкладки и панели при запуске
settings-features-persist-conversations = Сохранять диалоги агента в локальную историю
settings-features-show-sticky-command-header = Показывать закрепленный заголовок команды
settings-features-show-link-tooltip = Показывать всплывающую подсказку при клике на ссылки
settings-features-show-quit-warning = Показывать предупреждение перед выходом из приложения или аккаунта
settings-features-quit-on-last-window-closed = Выходить при закрытии всех окон
settings-features-show-changelog-after-update = Показывать уведомление со списком изменений после обновлений
settings-features-mouse-scroll-multiplier = Число строк прокрутки за один шаг колесика мыши
settings-features-auto-open-code-review = Автоматически открывать панель Code Review
settings-features-max-rows-per-block = Максимальное число строк в блоке
settings-features-ssh-wrapper = Zap SSH Wrapper
settings-features-ssh-auto-discovery = Автоматически обнаруживать SSH-хосты
settings-features-receive-desktop-notifications = Получать уведомления на рабочем столе от Zap
settings-features-show-in-app-agent-notifications = Показывать уведомления агента в приложении
settings-features-confirm-close-shared-session = Подтверждать перед закрытием сессии только для чтения
settings-features-global-hotkey-label = Глобальная горячая клавиша:
settings-features-global-hotkey-not-supported-on-wayland = Не поддерживается на Wayland.
settings-features-autocomplete-symbols = Автозакрытие кавычек, круглых и квадратных скобок
settings-features-error-underlining = Подчеркивание ошибок в командах
settings-features-syntax-highlighting = Подсветка синтаксиса команд
settings-features-completions-while-typing = Открывать меню автодополнения во время ввода
settings-features-command-corrections = Предлагать исправленные команды
settings-features-expand-aliases = Разворачивать алиасы при вводе
settings-features-middle-click-paste = Вставка средней кнопкой мыши
settings-features-vim-mode = Редактировать код и команды сочетаниями клавиш Vim
settings-features-at-context-menu = Включить контекстное меню «@» в режиме терминала
settings-features-slash-commands-in-terminal = Включить слэш-команды в режиме терминала
settings-features-outline-codebase-symbols = Строить структуру символов кодовой базы для контекстного меню «@»
settings-features-show-input-message-bar = Показывать строку сообщений ввода терминала
settings-features-show-autosuggestion-hint = Показывать подсказку сочетания клавиш для автопредложений
settings-features-show-autosuggestion-ignore = Показывать кнопку игнорирования автопредложений
settings-features-enable-mouse-reporting = Включить отчеты о событиях мыши
settings-features-enable-scroll-reporting = Включить отчеты о прокрутке
settings-features-enable-focus-reporting = Включить отчеты о фокусе
settings-features-use-audible-bell = Использовать звуковой сигнал
settings-features-double-click-smart-selection = Умное выделение двойным кликом
settings-features-show-help-block-in-new-sessions = Показывать блок справки в новых сессиях
settings-features-copy-on-select = Копировать при выделении
settings-features-show-global-workflows-in-command-search = Показывать глобальные Workflows в поиске команд (ctrl-r)
settings-features-linux-selection-clipboard = Учитывать буфер выделения Linux
settings-features-prefer-low-power-gpu = Предпочитать рендеринг новых окон на интегрированной графике (низкое энергопотребление)
settings-features-use-wayland = Использовать Wayland для управления окнами
settings-features-use-wayland-tooltip = Включает использование Wayland
settings-features-ctrl-tab-behavior-label = Поведение Ctrl+Tab:
settings-features-extra-meta-key-left-mac = Левая клавиша Option работает как Meta
settings-features-extra-meta-key-right-mac = Правая клавиша Option работает как Meta
settings-features-extra-meta-key-left-other = Левая клавиша Alt работает как Meta
settings-features-extra-meta-key-right-other = Правая клавиша Alt работает как Meta
settings-features-default-shell-header = Shell по умолчанию для новых сессий
settings-features-working-directory-header = Рабочий каталог для новых сессий
settings-features-notify-agent-task-completed = Уведомлять, когда агент завершает задачу
settings-features-notify-needs-attention = Уведомлять, когда команде или агенту требуется ваше внимание для продолжения
settings-features-play-notification-sounds = Воспроизводить звуки уведомлений
settings-features-default-session-mode = Режим по умолчанию для новых сессий
settings-features-block-rows-description = Установка лимита выше 100 тыс. строк может повлиять на производительность. Максимальное поддерживаемое число строк — { $max_rows }.
settings-features-toast-duration-label = Всплывающие уведомления остаются видимыми
settings-features-tab-key-behavior = Поведение клавиши Tab
settings-features-graphics-backend-label = Предпочитаемый графический бэкенд
settings-features-graphics-backend-current = Текущий бэкенд: { $backend }
settings-features-working-dir-home = Домашний каталог
settings-features-working-dir-previous = Каталог предыдущей сессии
settings-features-working-dir-custom = Свой каталог
settings-features-undo-close-enable = Включить повторное открытие закрытых сессий
settings-features-undo-close-grace-period = Льготный период (в секундах)
settings-features-configure-global-hotkey = Настроить глобальную горячую клавишу
settings-features-make-default-terminal = Сделать Zap терминалом по умолчанию
settings-features-pin-top = Закрепить сверху
settings-features-pin-bottom = Закрепить снизу
settings-features-pin-left = Закрепить слева
settings-features-pin-right = Закрепить справа
settings-features-default-option = По умолчанию
settings-features-tab-behavior-completions = Открывать меню автодополнения
settings-features-tab-behavior-autosuggestions = Принимать автоподсказку
settings-features-tab-behavior-user-defined = Пользовательский
settings-features-new-tab-placement-all = После всех вкладок
settings-features-new-tab-placement-current = После текущей вкладки
settings-features-width-percent = Ширина, %
settings-features-height-percent = Высота, %
settings-features-autohide-on-focus-loss = Автоматически скрывается при потере фокуса клавиатуры
settings-features-long-running-prefix = Если команда выполняется дольше
settings-features-long-running-suffix = секунд
settings-features-keybinding-label = Сочетание клавиш
settings-features-click-set-global-hotkey = Нажмите, чтобы задать глобальную горячую клавишу
settings-features-cancel = Отмена
settings-features-save = Сохранить
settings-features-press-new-shortcut = Нажмите новое сочетание клавиш
settings-features-change-keybinding = Изменить сочетание клавиш
settings-features-active-screen = Активный экран
settings-features-wayland-window-restore-warning = Положения окон не будут восстановлены на Wayland.
settings-features-see-docs = См. документацию.
settings-features-allowed-values-1-20 = Допустимые значения: 1-20
settings-features-supports-floating-1-20 = Поддерживаются дробные значения от 1 до 20.
settings-features-auto-open-code-review-description = Когда эта настройка включена, панель Code Review открывается при первом принятом diff в диалоге
settings-features-default-terminal-current = Zap — терминал по умолчанию
settings-features-takes-effect-new-sessions = Изменение вступит в силу в новых сессиях
settings-features-seconds = секунд
settings-features-vim-system-clipboard = Использовать безымянный регистр как системный буфер обмена
settings-features-vim-status-bar = Показывать строку состояния Vim
settings-features-tab-behavior-right-arrow-accepts = → принимает автоподсказки.
settings-features-tab-behavior-key-accepts = { $keybinding } принимает автоподсказки.
settings-features-completions-open-while-typing-sentence = Автодополнение открывается во время ввода.
settings-features-completions-open-while-typing-or-key = Автодополнение открывается во время ввода (или { $keybinding }).
settings-features-open-completions-unbound = Открытие меню автодополнения не назначено.
settings-features-tab-behavior-key-opens-completions = { $keybinding } открывает меню автодополнения.
settings-features-word-characters-label = Символы, считающиеся частью слова
settings-features-new-tab-placement = Размещение новой вкладки
settings-features-linux-selection-clipboard-tooltip = Следует ли поддерживать первичный буфер обмена Linux.
settings-features-changes-apply-new-windows = Изменения применятся к новым окнам.
settings-features-wayland-description = Включение этой настройки отключает поддержку глобальной горячей клавиши. Когда настройка выключена, текст может быть размытым, если ваш Wayland-композитор использует дробное масштабирование (напр., 125%).
settings-features-restart-warp-to-apply = Перезапустите Zap, чтобы изменения вступили в силу.

# --- ANCHOR-SUB-SETTINGS-PAGE-NAV (agent-settings-page-nav) ---
# 此锚点下放 settings_view/{settings_page,nav,delete_environment_confirmation_dialog,directory_color_add_picker,pane_manager}.rs 字符串
# 命名前缀:settings-page-* / settings-nav-* / settings-confirm-* / settings-color-picker-*

# ---- settings_page.rs ----
settings-page-info-icon-tooltip = Нажмите, чтобы узнать больше в документации
settings-page-local-only-icon-tooltip = Эта настройка не синхронизируется с вашими другими устройствами
settings-page-reset-to-default = Сбросить к значению по умолчанию

# ---- delete_environment_confirmation_dialog.rs ----
settings-confirm-cancel = Отмена
settings-confirm-delete-environment-button = Удалить окружение
settings-confirm-delete-environment-title = Удалить окружение?
settings-confirm-delete-environment-description = Вы уверены, что хотите удалить окружение { $name }?

# ---- directory_color_add_picker.rs ----
settings-color-picker-add-directory-footer = + Добавить каталог…
settings-color-picker-add-directory-color = Добавить цвет каталога

# ---- settings_file_footer.rs ----
settings-footer-open-file = Открыть файл настроек
settings-footer-alert-open-file = Открыть файл
settings-footer-alert-fix-with-oz = Исправить с помощью Oz

# --- ANCHOR-SUB-CODE (agent-settings-code) ---
settings-code-auto-open-review-panel = Автоматически открывать панель Code Review
settings-code-auto-open-review-panel-desc = Когда эта настройка включена, панель Code Review открывается при первом принятом diff в диалоге
settings-code-show-code-review-button = Показывать кнопку Code Review
settings-code-show-code-review-button-desc = Показывать кнопку в правом верхнем углу окна для переключения панели Code Review.
settings-code-show-diff-stats = Показывать статистику diff на кнопке Code Review
settings-code-show-diff-stats-desc = Показывать количество добавленных и удаленных строк на кнопке Code Review.
settings-code-project-explorer = Обозреватель проекта
settings-code-project-explorer-desc = Добавляет в панель инструментов слева обозреватель проекта / дерево файлов в стиле IDE.
settings-code-global-search = Глобальный поиск по файлам
settings-code-global-search-desc = Добавляет глобальный поиск по файлам в панель инструментов слева.

# --- ANCHOR-SUB-EXEC-MODAL-BLOCKS (agent-settings-misc) ---
# ---- execution_profile_view ----
settings-exec-profile-edit-button = Редактировать
settings-exec-profile-auto = Авто
settings-exec-profile-section-models = МОДЕЛИ
settings-exec-profile-section-permissions = РАЗРЕШЕНИЯ
settings-exec-profile-base-model = Базовая модель:
settings-exec-profile-full-terminal-use = Полное использование терминала:
settings-exec-profile-title-model = Генерация заголовков:
settings-exec-profile-active-ai-model = Активный AI:
settings-exec-profile-next-command-model = Следующая команда:
settings-exec-profile-computer-use = Computer use:
settings-exec-profile-apply-code-diffs = Применение diff к коду:
settings-exec-profile-read-files = Чтение файлов:
settings-exec-profile-execute-commands = Выполнение команд:
settings-exec-profile-interact-running-commands = Взаимодействие с запущенными командами:
settings-exec-profile-ask-questions = Задавать вопросы:
settings-exec-profile-call-mcp-servers = Вызов MCP-серверов:
settings-exec-profile-call-web-tools = Вызов веб-инструментов:
settings-exec-profile-chips-none = Нет
settings-exec-profile-perm-agent-decides = Агент решает
settings-exec-profile-perm-always-allow = Всегда разрешать
settings-exec-profile-perm-always-ask = Всегда спрашивать
settings-exec-profile-perm-unknown = Неизвестно
settings-exec-profile-perm-ask-on-first-write = Спросить при первой записи
settings-exec-profile-perm-never = Никогда
settings-exec-profile-perm-never-ask = Никогда не спрашивать
settings-exec-profile-perm-ask-unless-auto-approve = Спрашивать, если нет автоподтверждения
settings-exec-profile-perm-on = Вкл
settings-exec-profile-perm-off = Выкл
settings-exec-profile-directory-allowlist = Белый список каталогов:
settings-exec-profile-command-allowlist = Белый список команд:
settings-exec-profile-command-denylist = Черный список команд:
settings-exec-profile-mcp-allowlist = Белый список MCP:
settings-exec-profile-mcp-denylist = Черный список MCP:

# ---- execution_profile_editor (Profile Editor pane) ----
settings-exec-profile-editor-header = Редактор профиля
settings-exec-profile-editor-title = Редактирование профиля
settings-exec-profile-editor-name-label = Название
settings-exec-profile-editor-default-name-info = Название профиля по умолчанию нельзя изменить.
settings-exec-profile-editor-workspace-override-tooltip = Этот параметр задан настройками вашей организации и не может быть изменен.
settings-exec-profile-editor-section-models = МОДЕЛИ
settings-exec-profile-editor-section-permissions = РАЗРЕШЕНИЯ
settings-exec-profile-editor-base-model = Базовая модель
settings-exec-profile-editor-base-model-desc = Эта модель — основной движок агента. Она обеспечивает большинство взаимодействий и при необходимости вызывает другие модели для таких задач, как планирование или генерация кода. Zap может автоматически переключаться на другие модели в зависимости от их доступности или для вспомогательных задач, например суммаризации диалогов.
settings-exec-profile-editor-full-terminal-use-model = Модель полного использования терминала
settings-exec-profile-editor-full-terminal-use-model-desc = Модель, которая используется, когда агент работает внутри интерактивных терминальных приложений, таких как оболочки баз данных, отладчики, REPL или dev-серверы, — считывая живой вывод и записывая команды в PTY.
settings-exec-profile-editor-title-model = Модель генерации заголовков
settings-exec-profile-editor-title-model-desc = Модель, используемая для генерации кратких заголовков диалогов. По умолчанию — базовая модель; выберите здесь более дешевую и быструю модель, чтобы экономить токены на генерации заголовков и не влиять на основные рассуждения агента.
settings-exec-profile-editor-active-ai-model = Модель активного AI
settings-exec-profile-editor-active-ai-model-desc = Модель, которую используют проактивные функции AI: подсказки промптов после завершения команды, автодополнение на естественном языке в поле ввода агента и ранжирование релевантности кодовой базы. По умолчанию — базовая модель; выберите небольшую быструю модель, чтобы эти функции оставались отзывчивыми, не влияя на основные рассуждения агента.
settings-exec-profile-editor-next-command-model = Модель следующей команды
settings-exec-profile-editor-next-command-model-desc = Модель, используемая для предсказания следующей команды shell (серая встроенная автоподсказка + подсказка на стартовом экране после завершения блока). Чувствительна к задержкам — выберите самую маленькую и быструю модель BYOP из доступных. По умолчанию — базовая модель.
settings-exec-profile-editor-computer-use-model = Модель Computer use
settings-exec-profile-editor-computer-use-model-desc = Модель, которая используется, когда агент берет управление вашим компьютером для взаимодействия с графическими приложениями с помощью движений мыши, кликов и ввода с клавиатуры.
settings-exec-profile-editor-apply-code-diffs = Применение diff к коду
settings-exec-profile-editor-read-files = Чтение файлов
settings-exec-profile-editor-execute-commands = Выполнение команд
settings-exec-profile-editor-interact-running-commands = Взаимодействие с запущенными командами
settings-exec-profile-editor-computer-use = Computer use
settings-exec-profile-editor-ask-questions = Задавать вопросы
settings-exec-profile-editor-call-mcp-servers = Вызов MCP-серверов
settings-exec-profile-editor-call-web-tools = Вызов веб-инструментов
settings-exec-profile-editor-call-web-tools-desc = Агент может использовать веб-поиск, когда это помогает выполнить задачу.
settings-exec-profile-editor-directory-allowlist = Белый список каталогов
settings-exec-profile-editor-directory-allowlist-desc = Дает агенту доступ к файлам в определенных каталогах.
settings-exec-profile-editor-command-allowlist = Белый список команд
settings-exec-profile-editor-command-allowlist-desc = Регулярные выражения для сопоставления команд, которые Oz может выполнять автоматически.
settings-exec-profile-editor-command-denylist = Черный список команд
settings-exec-profile-editor-command-denylist-desc = Регулярные выражения для сопоставления команд, на выполнение которых Oz всегда должен запрашивать разрешение.
settings-exec-profile-editor-mcp-allowlist = Белый список MCP
settings-exec-profile-editor-mcp-allowlist-desc = MCP-серверы, которые Oz может вызывать.
settings-exec-profile-editor-mcp-denylist = Черный список MCP
settings-exec-profile-editor-mcp-denylist-desc = MCP-серверы, которые Oz не может вызывать.

# ---- agent_assisted_environment_modal ----
settings-env-modal-add-repo = Добавить репозиторий
settings-env-modal-cancel = Отмена
settings-env-modal-create-environment = Создать окружение
settings-env-modal-selected-repos = Выбранные репозитории
settings-env-modal-no-repos-selected = Репозитории пока не выбраны
settings-env-modal-available-repos = Доступные проиндексированные репозитории
settings-env-modal-loading = Загрузка локально проиндексированных репозиториев…
settings-env-modal-empty-no-indexed = Локально проиндексированные репозитории пока не найдены. Проиндексируйте репозиторий и попробуйте еще раз.
settings-env-modal-unavailable-build = Выбор локальных репозиториев недоступен в этой сборке.
settings-env-modal-all-selected = Все локально проиндексированные репозитории уже выбраны.
settings-env-modal-unknown-repo-name = (неизвестно)
settings-env-modal-not-git-repo = Выбранная папка не является Git-репозиторием: { $path }
settings-env-modal-no-directory-selected = Каталог не выбран
settings-env-modal-dialog-title = Выберите репозитории для вашего окружения
settings-env-modal-dialog-description-indexed = Выберите локально проиндексированные репозитории, чтобы задать контекст для агента, создающего окружение.
settings-env-modal-dialog-description-default = Выберите репозитории, чтобы задать контекст для агента, создающего окружение.

# ---- show_blocks_view ----
settings-show-blocks-page-title = Общие блоки
settings-show-blocks-unshare-menu-item = Отменить общий доступ
settings-show-blocks-copy-link = Копировать ссылку
settings-show-blocks-deleting = Удаление…
settings-show-blocks-executed-on = Выполнен: { $time }
settings-show-blocks-empty = У вас пока нет общих блоков.
settings-show-blocks-loading = Получение блоков…
settings-show-blocks-load-failed = Не удалось загрузить блоки. Попробуйте еще раз.
settings-show-blocks-link-copied = Ссылка скопирована.
settings-show-blocks-unshare-success = Общий доступ к блоку успешно отменен.
settings-show-blocks-unshare-failed = Не удалось отменить общий доступ к блоку. Попробуйте еще раз.
settings-show-blocks-confirm-dialog-title = Отменить общий доступ к блоку
settings-show-blocks-confirm-dialog-text = Вы уверены, что хотите отменить общий доступ к этому блоку?

    Он больше не будет доступен по ссылке и будет навсегда удален с серверов Zap.
settings-show-blocks-confirm-cancel = Отмена
settings-show-blocks-confirm-unshare = Отменить общий доступ

# --- ANCHOR-SUB-APPEARANCE (agent-settings-appearance) ---
# 此锚点下放 settings_view/appearance_page.rs 剩余字符串(不含已完成的 Language widget)
# 命名前缀:settings-appearance-*

# Categories
settings-appearance-category-themes = Темы
settings-appearance-category-language = Язык
settings-appearance-category-icon = Иконка
settings-appearance-category-window = Окно
settings-appearance-category-input = Ввод
settings-appearance-category-panes = Панели
settings-appearance-category-blocks = Блоки
settings-appearance-category-text = Текст
settings-appearance-category-cursor = Курсор
settings-appearance-category-tabs = Вкладки
settings-appearance-category-fullscreen-apps = Полноэкранные приложения

# Theme widget
settings-appearance-theme-create-custom = Создайте свою собственную тему
settings-appearance-theme-mode-light = Светлая
settings-appearance-theme-mode-dark = Темная
settings-appearance-theme-mode-current = Текущая тема
settings-appearance-theme-sync-os-label = Синхронизировать с ОС
settings-appearance-theme-sync-os-description = Автоматически переключаться между светлой и темной темами, когда это делает система.

# Custom App Icon widget
settings-appearance-custom-icon-label = Настройте иконку приложения
settings-appearance-custom-icon-bundle-warning = Для изменения иконки приложения требуется собранная версия приложения (bundle).
settings-appearance-custom-icon-restart-warning = Возможно, потребуется перезапустить Zap, чтобы MacOS применил выбранный стиль иконки.

# Window widgets
settings-appearance-window-custom-size-label = Открывать новые окна с заданным размером
settings-appearance-window-columns-label = Столбцы
settings-appearance-window-rows-label = Строки
settings-appearance-window-opacity-label = Прозрачность окна:
settings-appearance-window-opacity-value = Прозрачность окна: { $value }
settings-appearance-window-opacity-not-supported = Прозрачность не поддерживается вашими графическими драйверами.
settings-appearance-window-opacity-graphics-warning = Выбранные графические настройки могут не поддерживать отрисовку прозрачных окон.
settings-appearance-window-opacity-graphics-warning-hint = Попробуйте изменить настройки графического бэкенда или интегрированного GPU в разделе «Функции» > «Система».
settings-appearance-window-blur-radius = Радиус размытия окна: { $value }
settings-appearance-window-blur-texture-label = Использовать размытие окна (текстура Acrylic)
settings-appearance-window-blur-texture-not-supported = Выбранное оборудование может не поддерживать отрисовку прозрачных окон.
settings-appearance-tools-panel-consistent-label = Видимость панели инструментов одинакова во всех вкладках

# Input
settings-appearance-input-type-label = Тип ввода
settings-appearance-input-type-warp = Zap
settings-appearance-input-type-shell = Shell (PS1)
settings-appearance-input-position-label = Расположение поля ввода
settings-appearance-input-mode-pinned-bottom = Закрепить внизу (режим Zap)
settings-appearance-input-mode-pinned-top = Закрепить вверху (обратный режим)
settings-appearance-input-mode-waterfall = Начинать сверху (классический режим)

# Panes
settings-appearance-pane-dim-inactive-label = Затемнять неактивные панели
settings-appearance-pane-focus-follows-mouse-label = Фокус следует за мышью

# Blocks
settings-appearance-block-compact-label = Компактный режим
settings-appearance-block-jump-bottom-label = Показывать кнопку «Перейти к концу блока»
settings-appearance-block-show-dividers-label = Показывать разделители блоков

# Text / Fonts
settings-appearance-font-agent-label = Шрифт агента
settings-appearance-font-match-terminal = Как в терминале
settings-appearance-font-ui-label = Шрифт интерфейса
settings-appearance-font-terminal-label = Шрифт терминала
settings-appearance-font-terminal-fallback-label = Резервный шрифт
settings-appearance-font-fallback-system = Системный резервный
settings-appearance-font-view-all-system = Показать все доступные системные шрифты
settings-appearance-font-weight-label = Насыщенность шрифта
settings-appearance-font-size-label = Размер шрифта (px)
settings-appearance-font-line-height-label = Межстрочный интервал
settings-appearance-font-reset-default = Сбросить к значению по умолчанию
settings-appearance-font-notebook-size-label = Размер шрифта блокнота
settings-appearance-markdown-heading-scale-label = Масштаб шрифта заголовков Markdown
settings-appearance-markdown-heading-scale-description = Масштаб задается относительно размера моноширинного шрифта (терминала). Итоговый размер = размер моноширинного шрифта × масштаб
settings-appearance-markdown-heading-h1-label = Масштаб H1
settings-appearance-markdown-heading-h2-label = Масштаб H2
settings-appearance-markdown-heading-h3-label = Масштаб H3
settings-appearance-markdown-heading-h4-label = Масштаб H4
settings-appearance-markdown-heading-h5-label = Масштаб H5
settings-appearance-markdown-heading-h6-label = Масштаб H6
settings-appearance-font-thin-strokes-label = Использовать тонкие штрихи
settings-appearance-font-thin-strokes-never = Никогда
settings-appearance-font-thin-strokes-low-dpi = На экранах с низким DPI
settings-appearance-font-thin-strokes-high-dpi = На экранах с высоким DPI
settings-appearance-font-thin-strokes-always = Всегда
settings-appearance-font-min-contrast-label = Обеспечивать минимальный контраст
settings-appearance-font-min-contrast-always = Всегда
settings-appearance-font-min-contrast-named-only = Только для именованных цветов
settings-appearance-font-min-contrast-never = Никогда
settings-appearance-font-ligatures-label = Показывать лигатуры в терминале
settings-appearance-font-ligatures-perf-tooltip = Лигатуры могут снижать производительность

# Cursor
settings-appearance-cursor-type-label = Тип курсора
settings-appearance-cursor-disabled-vim = Тип курсора отключен в режиме Vim
settings-appearance-cursor-blink-label = Мигающий курсор

# Tabs
settings-appearance-tab-close-position-label = Расположение кнопки закрытия вкладки
settings-appearance-tab-close-position-right = Справа
settings-appearance-tab-close-position-left = Слева
settings-appearance-tab-show-indicators-label = Показывать индикаторы вкладок
settings-appearance-tab-show-code-review-label = Показывать кнопку ревью кода
settings-appearance-tab-preserve-active-color-label = Сохранять цвет активной вкладки для новых вкладок
settings-appearance-tab-vertical-layout-label = Использовать вертикальное расположение вкладок
settings-appearance-tab-show-vertical-panel-in-restored-windows-label = Показывать панель вертикальных вкладок в восстановленных окнах
settings-appearance-tab-show-vertical-panel-in-restored-windows-description = Если включено, при повторном открытии или восстановлении окна открывается панель вертикальных вкладок, даже если она была закрыта в момент последнего сохранения окна.
settings-appearance-tab-show-title-bar-search-bar-label = Показывать строку поиска в строке заголовка
settings-appearance-tab-show-title-bar-search-bar-description = Показывать строку поиска «Поиск сессий, агентов, файлов…» в центре строки заголовка; щелчок по ней открывает палитру команд. Отключите, чтобы оставить это место пустым. Применяется только при вертикальном расположении вкладок.
workspace-title-bar-search-placeholder = Поиск сессий, агентов, файлов…
settings-appearance-tab-use-prompt-as-title-label = Использовать последний промпт пользователя как заголовок диалога в названиях вкладок
settings-appearance-tab-use-prompt-as-title-description = Показывать последний промпт пользователя вместо сгенерированного заголовка диалога для сессий встроенного AI и сторонних агентов при вертикальном расположении вкладок.
settings-appearance-tab-toolbar-layout-label = Расположение панели инструментов заголовка
settings-appearance-tab-directory-colors-label = Цвета вкладок по директориям
settings-appearance-tab-directory-colors-description = Автоматически раскрашивать вкладки в зависимости от директории или репозитория, в котором вы работаете.
settings-appearance-tab-directory-color-default-tooltip = По умолчанию (без цвета)
settings-appearance-zen-mode-label = Показывать панель вкладок
settings-appearance-zen-decoration-always = Всегда
settings-appearance-zen-decoration-windowed = В оконном режиме
settings-appearance-zen-decoration-on-hover = Только при наведении

# Full-screen apps
settings-appearance-alt-screen-padding-label = Использовать пользовательские отступы в альтернативном экране
settings-appearance-alt-screen-uniform-padding-label = Одинаковые отступы (px)

# Zoom
settings-appearance-zoom-label = Масштаб
settings-appearance-zoom-secondary = Задает уровень масштаба по умолчанию для всех окон

# --- ANCHOR-SUB-ENVIRONMENTS (agent-settings-environments) ---
settings-environments-page-title = Окружения
settings-environments-page-description = Окружения определяют, где выполняются ваши фоновые агенты. Настройте окружение за несколько минут через GitHub (рекомендуется), с помощью Zap или вручную.
settings-environments-search-placeholder = Поиск окружений…
settings-environments-no-matches = Ни одно окружение не соответствует вашему запросу.
settings-environments-section-personal = Личные
settings-environments-section-team-default = Предоставлены Zap и этим устройством
settings-environments-section-team-named = Общие для Zap и { $team }
settings-environments-env-id-prefix = ID окружения: { $id }
settings-environments-detail-image = Образ: { $image }
settings-environments-detail-repos = Репозитории: { $repos }
settings-environments-detail-setup-commands = Команды настройки: { $commands }
settings-environments-last-edited = Последнее изменение: { $time }
settings-environments-last-used = Последнее использование: { $time }
settings-environments-last-used-never = Последнее использование: никогда
settings-environments-view-my-runs = Показать мои запуски
settings-environments-tooltip-share = Поделиться
settings-environments-tooltip-edit = Изменить
settings-environments-empty-header = Вы еще не настроили ни одного окружения.
settings-environments-empty-subheader = Выберите, как настроить окружение:
settings-environments-empty-quick-setup-title = Быстрая настройка
settings-environments-empty-suggested-badge = Рекомендуем
settings-environments-empty-quick-setup-subtitle = Выберите репозитории GitHub, с которыми хотите работать, и мы предложим базовый образ и конфигурацию
settings-environments-empty-use-agent-title = Использовать агента
settings-environments-empty-use-agent-subtitle = Выберите локально настроенный проект, и мы поможем создать на его основе окружение
settings-environments-button-loading = Загрузка…
settings-environments-button-retry = Повторить
settings-environments-button-authorize = Авторизовать
settings-environments-button-get-started = Начать
settings-environments-button-launch-agent = Запустить агента
settings-environments-toast-update-success = Окружение успешно обновлено
settings-environments-toast-create-success = Окружение успешно создано
settings-environments-toast-delete-success = Окружение успешно удалено
settings-environments-toast-share-success = Окружение успешно добавлено в общий доступ
settings-environments-toast-share-failure = Не удалось поделиться окружением с командой
settings-environments-toast-create-not-logged-in = Не удалось создать окружение: вы не вошли в систему.
settings-environments-toast-save-not-found = Не удалось сохранить: окружение больше не существует.
settings-environments-toast-share-no-team = Не удалось поделиться окружением: вы сейчас не состоите в команде.
settings-environments-toast-share-not-synced = Не удалось поделиться окружением: окружение еще не синхронизировано.
settings-update-environment-name-placeholder = Название окружения
settings-update-environment-docker-image-placeholder = например, python:3.11, node:20-alpine
settings-update-environment-repos-placeholder-authed = Введите репозитории (в формате owner/repo)
settings-update-environment-repos-placeholder-unauthenticated = Вставьте URL репозиториев
settings-update-environment-setup-command-placeholder = например, cd my-repo && pip install -r requirements.txt
settings-update-environment-description-placeholder = например, это окружение предназначено для всех агентов, работающих с фронтендом

# --- ANCHOR-SUB-AGENT-PROVIDERS (agent-settings-agent-providers) ---
# 此锚点下放 settings_view/agent_providers_widget.rs 字符串
# 命名前缀:settings-agent-providers-*
settings-agent-providers-title = Провайдеры агентов
settings-agent-providers-description = Настройте собственные провайдеры агентов для разных протоколов — OpenAI-совместимые (DeepSeek, Zhipu GLM, Moonshot, DashScope, SiliconFlow, OpenRouter и другие), Anthropic, Gemini и локальный Ollama. Модели можно добавлять вручную (сопоставление отображаемого имени и ID модели) или получать автоматически из API. Метаданные провайдеров хранятся в локальном файле settings.toml; ключи API надежно хранятся в системном хранилище ключей.
settings-agent-providers-empty = Провайдеры еще не настроены. Нажмите [+ Добавить провайдера] в правом верхнем углу, чтобы добавить нового.
settings-agent-providers-add-button = + Добавить провайдера
settings-agent-providers-search-placeholder = Поиск провайдеров…
settings-agent-providers-quick-add-title = Быстрое добавление
settings-agent-providers-refresh-catalog = Обновить каталог
settings-agent-providers-loading-catalog = Загрузка каталога models.dev… (первая загрузка может занять несколько секунд)
settings-agent-providers-catalog-empty = Каталог models.dev пуст. Нажмите [Обновить каталог], чтобы повторить попытку.
settings-agent-providers-catalog-load-failed = Не удалось загрузить каталог models.dev. Нажмите [Обновить каталог], чтобы повторить попытку.
settings-agent-providers-no-match = Нет совпадений с «{ $query }»
settings-agent-providers-collapse = Свернуть ▲
settings-agent-providers-expand-remaining = Развернуть оставшиеся { $count } ▼
settings-agent-providers-row-missing = (для этого провайдера пока не привязаны редакторы: { $id })
settings-agent-providers-field-name = Название
settings-agent-providers-field-base-url = Базовый URL
settings-agent-providers-field-api-key = Ключ API
settings-agent-providers-field-api-type = Тип API
settings-agent-providers-api-type-hint = (genai использует это значение, чтобы явно привязать адаптер и избежать ложного распознавания по имени модели. Если базовый URL пуст, будет использован URL по умолчанию: { $url })
settings-agent-providers-name-placeholder = Название провайдера (например, DeepSeek, локальный Ollama)
settings-agent-providers-api-key-placeholder = sk-... (необязательно, оставьте пустым для локальных провайдеров вроде ollama)
settings-agent-providers-models-label = Модели ({ $count })
settings-agent-providers-models-empty-hint = Модели пока не настроены. Нажмите [+ Добавить модель], чтобы добавить вручную, или [Получить из API], чтобы получить автоматически.
settings-agent-providers-models-header-name = Отображаемое имя
settings-agent-providers-models-header-id = ID модели
settings-agent-providers-models-header-context = Контекст (ток)
settings-agent-providers-models-header-output = Вывод (ток)
settings-agent-providers-model-name-placeholder = Отображаемое имя (например, DS-V3 General)
settings-agent-providers-model-id-placeholder = ID модели (поле `model`, отправляемое в API, например deepseek-chat)
settings-agent-providers-model-context-placeholder = Контекст (токены)
settings-agent-providers-model-output-placeholder = Вывод (токены)
settings-agent-providers-add-model = + Добавить модель
settings-agent-providers-fetch-from-api = Получить из API
settings-agent-providers-sync-models-dev = Синхронизировать с models.dev
settings-agent-providers-remove = Удалить
settings-agent-providers-save = Сохранить
settings-agent-providers-saved-toast = Сохранено

# ---- AI page (settings_view/ai_page.rs) ----
settings-ai-title = AI
settings-ai-active-ai = Активный AI
settings-ai-input-autodetection = автоопределение команд терминала в поле ввода агента
settings-ai-input-autodetection-legacy = определение естественного языка
settings-ai-next-command-description = Позвольте AI предлагать следующую команду для запуска на основе истории ваших команд, результатов их выполнения и типичных Workflow.
settings-ai-prompt-suggestions-description = Позвольте AI предлагать промпты на естественном языке в виде встроенных баннеров в поле ввода на основе недавних команд и их результатов.
settings-ai-suggested-code-banners-description = Позвольте AI предлагать диффы кода и запросы в виде встроенных баннеров в списке блоков на основе недавних команд и их результатов.
settings-ai-natural-language-autosuggestions = Позвольте AI предлагать автоподсказки на естественном языке на основе недавних команд и их результатов.
settings-ai-git-operations-autogen-description = Позвольте AI генерировать сообщения коммитов, а также заголовки и описания pull request'ов.

# =============================================================================
# SECTION: ai (Owner: agent-ai)
# Files: app/src/ai/**, app/src/ai_assistant/**
# =============================================================================

# (placeholder — to be filled by agent-ai)

# =============================================================================
# SECTION: command-palette (Owner: agent-cmdpal)
# Files: app/src/command_palette.rs, app/src/palette/**
# =============================================================================

# (placeholder)

# =============================================================================
# SECTION: drive (Owner: agent-drive)
# Files: app/src/drive/**
# =============================================================================

# (placeholder)

# =============================================================================
# SECTION: workspace (Owner: agent-workspace)
# Files: app/src/workspace/**, app/src/workspaces/**
# =============================================================================

# (placeholder)

# =============================================================================
# SECTION: modal (Owner: agent-modal)
# Files: app/src/modal/**, app/src/prompt/**, app/src/quit_warning/**
# =============================================================================

# (placeholder)

# =============================================================================
# SECTION: auth (Owner: agent-auth)
# Files: app/src/auth/**
# =============================================================================

# (placeholder)

# =============================================================================
# SECTION: banner (Owner: agent-banner)
# Files: app/src/banner/**
# =============================================================================

banner-dont-show-again = Больше не показывать

# =============================================================================
# SECTION: quit-warning (Owner: agent-quit-warning)
# Files: app/src/quit_warning/mod.rs
# =============================================================================

# ---- Dialog titles ----
quit-warning-title-pane = Закрыть панель?
quit-warning-title-tab-singular = Закрыть вкладку?
quit-warning-title-tab-plural = Закрыть вкладки?
quit-warning-title-window = Закрыть окно?
quit-warning-title-app = Выйти из Zap?
quit-warning-title-editor-tab = Сохранить изменения?

# ---- Buttons ----
quit-warning-button-confirm-close = Да, закрыть
quit-warning-button-confirm-quit = Да, выйти
quit-warning-button-save = Сохранить
quit-warning-button-discard = Не сохранять
quit-warning-button-show-processes = Показать запущенные процессы
quit-warning-button-cancel = Отмена

# ---- Warning body lines ----
# Suffix appended to each warning line, indicating the scope.
quit-warning-suffix-tab = { " " }в этой вкладке.
quit-warning-suffix-window = { " " }в этом окне.
quit-warning-suffix-pane = { " " }в этой панели.
quit-warning-suffix-default = .

# Process info: "{count} process(es) running" with optional window/tab qualifier.
quit-warning-processes-running = У вас выполняется { $count } { $count ->
        [one] процесс
        [few] процесса
        [many] процессов
       *[other] процесса
    }
quit-warning-processes-in-windows = { " " }в { $count } окнах
quit-warning-processes-in-tabs = { " " }в { $count } вкладках

# Shared sessions line.
quit-warning-shared-sessions = Вы делитесь { $count } { $count ->
        [one] сессией
        [few] сессиями
        [many] сессиями
       *[other] сессией
    }

# Unsaved code changes (generic scope).
quit-warning-unsaved-changes = У вас есть несохраненные изменения в файлах

# Unsaved code changes for a specific editor tab.
quit-warning-unsaved-editor-tab = Хотите сохранить изменения, внесенные в { $file }? Если вы не сохраните их, изменения будут потеряны.
quit-warning-unsaved-editor-tab-fallback-name = этот файл

# --- ANCHOR-SUB-RULES-PAGE (agent-rules-page) ---
# Manage Rules 页面(Zap Drive 中的 AI Fact Collection)。
rules-collection-name = Правила

# --- ANCHOR-SUB-KEYBINDING-DESC (agent-keybinding-descriptions) ---
# Description 文案 for keyboard binding entries shown in the Settings >
# Keyboard Shortcuts page and the command palette. Each key corresponds to
# a binding registered via `EditableBinding::new(name, description, action)`
# or `BindingDescription::new("…")`. The binding `name` (e.g.
# `workspace:open_settings_file`) is **not** translated — it is a protocol
# field used to persist user-customised shortcuts.

# Tabs / sessions
keybinding-desc-workspace-cycle-next-session = Переключиться на следующую вкладку
keybinding-desc-workspace-cycle-prev-session = Переключиться на предыдущую вкладку
keybinding-desc-workspace-add-window = Создать новое окно
keybinding-desc-workspace-new-file = Новый файл
keybinding-desc-workspace-zoom-in = Увеличить масштаб
keybinding-desc-workspace-zoom-out = Уменьшить масштаб
keybinding-desc-workspace-reset-zoom = Сбросить масштаб
keybinding-desc-workspace-increase-font-size = Увеличить размер шрифта
keybinding-desc-workspace-decrease-font-size = Уменьшить размер шрифта
keybinding-desc-workspace-reset-font-size = Сбросить размер шрифта до значения по умолчанию
keybinding-desc-workspace-increase-zoom = Увеличить уровень масштаба
keybinding-desc-workspace-decrease-zoom = Уменьшить уровень масштаба
keybinding-desc-workspace-reset-zoom-level = Сбросить уровень масштаба до значения по умолчанию
keybinding-desc-workspace-save-launch-config = Сохранить новую конфигурацию запуска

# Project Explorer / panels
keybinding-desc-workspace-toggle-project-explorer = Показать/скрыть проводник проектов
keybinding-desc-workspace-toggle-project-explorer-menu = Проводник проектов
keybinding-desc-workspace-show-theme-chooser = Открыть выбор темы
keybinding-desc-workspace-toggle-tab-configs-menu = Открыть меню конфигураций вкладок

# Switch to N-th tab
keybinding-desc-workspace-activate-1st-tab = Переключиться на 1-ю вкладку
keybinding-desc-workspace-activate-2nd-tab = Переключиться на 2-ю вкладку
keybinding-desc-workspace-activate-3rd-tab = Переключиться на 3-ю вкладку
keybinding-desc-workspace-activate-4th-tab = Переключиться на 4-ю вкладку
keybinding-desc-workspace-activate-5th-tab = Переключиться на 5-ю вкладку
keybinding-desc-workspace-activate-6th-tab = Переключиться на 6-ю вкладку
keybinding-desc-workspace-activate-7th-tab = Переключиться на 7-ю вкладку
keybinding-desc-workspace-activate-8th-tab = Переключиться на 8-ю вкладку
keybinding-desc-workspace-activate-last-tab = Переключиться на последнюю вкладку
keybinding-desc-workspace-activate-prev-tab = Активировать предыдущую вкладку
keybinding-desc-workspace-activate-next-tab = Активировать следующую вкладку

# Pane navigation
keybinding-desc-pane-group-navigate-prev = Активировать предыдущую панель
keybinding-desc-pane-group-navigate-next = Активировать следующую панель

# Mouse / Notebooks / Workflows / Folders
keybinding-desc-workspace-toggle-mouse-reporting = Включить/выключить события мыши
keybinding-desc-workspace-create-personal-notebook = Создать новый личный блокнот
keybinding-desc-workspace-create-personal-notebook-menu = Новый личный блокнот
keybinding-desc-workspace-create-personal-workflow = Создать новый личный Workflow
keybinding-desc-workspace-create-personal-workflow-menu = Новый личный Workflow
keybinding-desc-workspace-create-personal-folder = Создать новую личную папку
keybinding-desc-workspace-create-personal-folder-menu = Новая личная папка

# New tab variants
keybinding-desc-workspace-new-tab = Создать новую вкладку
keybinding-desc-workspace-new-terminal-tab = Новая вкладка терминала
keybinding-desc-workspace-new-agent-tab = Новая вкладка агента
keybinding-desc-workspace-new-cloud-agent-tab = Новая вкладка агента
new-session-create-new-tab = Создать новую вкладку
new-session-create-new-window = Создать новое окно
new-session-split-pane-down = Разделить панель вниз
new-session-split-pane-right = Разделить панель вправо
new-session-split-pane-up = Разделить панель вверх
new-session-split-pane-left = Разделить панель влево
new-session-create-new-tab-with-shell = Создать новую вкладку: { $shell }
new-session-create-new-window-with-shell = Создать новое окно: { $shell }
new-session-split-pane-with-shell = Разделить панель { $direction }: { $shell }
new-session-direction-down = Вниз
new-session-direction-right = Вправо
new-session-direction-up = Вверх
new-session-direction-left = Влево

# Left / right panel toggles
keybinding-desc-workspace-toggle-left-panel = Открыть левую панель
keybinding-desc-workspace-toggle-right-panel = Показать/скрыть ревью кода
keybinding-desc-workspace-toggle-right-panel-menu = Показать/скрыть ревью кода
keybinding-desc-workspace-toggle-vertical-tabs = Показать/скрыть панель вертикальных вкладок
keybinding-desc-workspace-toggle-vertical-tabs-menu = Показать/скрыть панель вертикальных вкладок
keybinding-desc-workspace-left-panel-agent-conversations = Левая панель: диалоги с агентами
keybinding-desc-workspace-left-panel-project-explorer = Левая панель: проводник проектов
keybinding-desc-workspace-left-panel-global-search = Левая панель: глобальный поиск
keybinding-desc-workspace-left-panel-warp-drive = Левая панель: Zap Drive
keybinding-desc-workspace-left-panel-ssh-manager = Левая панель: менеджер SSH
keybinding-desc-workspace-left-panel-skill-manager = Левая панель: менеджер навыков
keybinding-desc-workspace-open-global-search = Открыть глобальный поиск
keybinding-desc-workspace-open-global-search-menu = Глобальный поиск
keybinding-desc-workspace-toggle-warp-drive = Показать/скрыть Zap Drive
keybinding-desc-workspace-toggle-warp-drive-menu = Zap Drive
keybinding-desc-workspace-toggle-conversation-list-view = Показать/скрыть список диалогов с агентами
keybinding-desc-workspace-toggle-conversation-list-view-menu = Список диалогов с агентами
keybinding-desc-workspace-close-panel = Закрыть активную панель

# Command palette / navigation
keybinding-desc-workspace-toggle-command-palette = Показать/скрыть палитру команд
keybinding-desc-workspace-toggle-command-palette-menu = Палитра команд
keybinding-desc-workspace-toggle-navigation-palette = Показать/скрыть палитру навигации
keybinding-desc-workspace-toggle-navigation-palette-menu = Палитра навигации
keybinding-desc-workspace-toggle-launch-config-palette = Палитра конфигураций запуска
keybinding-desc-workspace-toggle-files-palette = Показать/скрыть палитру файлов
keybinding-desc-workspace-search-drive = Поиск в Zap Drive
keybinding-desc-workspace-move-tab-left = Переместить вкладку влево
keybinding-desc-workspace-move-tab-up = переместить вкладку вверх
keybinding-desc-workspace-move-tab-right = Переместить вкладку вправо
keybinding-desc-workspace-move-tab-down = переместить вкладку вниз

# Keybindings settings
keybinding-desc-workspace-toggle-keybindings-page = Переключить сочетания клавиш
keybinding-desc-workspace-show-keybinding-settings = Открыть редактор сочетаний клавиш
keybinding-desc-workspace-toggle-block-snackbar = Переключить закрепленный заголовок команды

# Window / tab close
keybinding-desc-workspace-rename-active-tab = Переименовать текущую вкладку
keybinding-desc-workspace-terminate-app = Выйти из Zap
keybinding-desc-workspace-close-window = Закрыть окно
keybinding-desc-workspace-close-active-tab = Закрыть текущую вкладку
keybinding-desc-workspace-close-other-tabs = Закрыть другие вкладки
keybinding-desc-workspace-close-tabs-right = Закрыть вкладки справа
keybinding-desc-workspace-close-tabs-below = закрыть вкладки снизу

# Notifications
keybinding-desc-workspace-toggle-notifications-on = Включить уведомления
keybinding-desc-workspace-toggle-notifications-off = Выключить уведомления

# Updates / changelog
keybinding-desc-workspace-update-and-relaunch = Установить обновление и перезапустить
keybinding-desc-workspace-check-for-updates = Проверить обновления
keybinding-desc-workspace-view-changelog = Показать последний журнал изменений

# Resource center / Drive export / CLI
keybinding-desc-workspace-toggle-resource-center = Переключить центр ресурсов
keybinding-desc-workspace-export-all-warp-drive-objects = Экспортировать все объекты Zap Drive
keybinding-desc-workspace-install-cli = Установить команду Oz CLI
keybinding-desc-workspace-uninstall-cli = Удалить команду Oz CLI

# AI assistant / agents
keybinding-desc-workspace-toggle-ai-assistant = Переключить Zap AI

# Env vars / prompts
keybinding-desc-workspace-create-personal-env-vars = Создать новые персональные переменные окружения
keybinding-desc-workspace-create-personal-env-vars-menu = Новые персональные переменные окружения
keybinding-desc-workspace-create-personal-ai-prompt = Создать новый персональный промпт
keybinding-desc-workspace-create-personal-ai-prompt-menu = Новый персональный промпт

# Focus / import
keybinding-desc-workspace-shift-focus-left = Переключить фокус на левую панель
keybinding-desc-workspace-shift-focus-right = Переключить фокус на правую панель
keybinding-desc-workspace-import-to-personal-drive = Импортировать в личный диск

# Drive / repository / AI rules / MCP
keybinding-desc-workspace-open-repository = Открыть репозиторий
keybinding-desc-workspace-open-repository-menu = Открыть репозиторий
keybinding-desc-workspace-open-ai-fact-collection = Открыть правила AI
keybinding-desc-workspace-open-mcp-servers = Открыть серверы MCP
keybinding-desc-workspace-jump-to-latest-toast = Перейти к последней задаче агента
keybinding-desc-workspace-toggle-notification-mailbox = Переключить почтовый ящик уведомлений

# Settings pages
keybinding-desc-workspace-show-settings = Открыть настройки
keybinding-desc-workspace-show-settings-menu = Настройки
# Zap: keybinding-desc-workspace-show-settings-account removed alongside the
# Account settings page.
keybinding-desc-workspace-show-settings-appearance = Открыть настройки: Внешний вид
keybinding-desc-workspace-show-settings-appearance-menu = Внешний вид…
keybinding-desc-workspace-show-settings-features = Открыть настройки: Функции
keybinding-desc-workspace-show-settings-shared-blocks = Открыть настройки: Общие блоки
keybinding-desc-workspace-show-settings-shared-blocks-menu = Просмотр общих блоков…
keybinding-desc-workspace-show-settings-keyboard-shortcuts = Открыть настройки: Сочетания клавиш
keybinding-desc-workspace-show-settings-keyboard-shortcuts-menu = Настроить сочетания клавиш…
keybinding-desc-workspace-show-settings-about = Открыть настройки: О программе
keybinding-desc-workspace-show-settings-about-menu = О Zap
keybinding-desc-workspace-show-settings-warpify = Открыть настройки: Warpify
keybinding-desc-workspace-show-settings-warpify-menu = Настроить Warpify…
keybinding-desc-workspace-show-settings-ai = Открыть настройки: AI
keybinding-desc-workspace-show-settings-code = Открыть настройки: Код
keybinding-desc-workspace-show-settings-referrals = Открыть настройки: Рефералы
keybinding-desc-workspace-show-settings-environments = Открыть настройки: Окружения
keybinding-desc-workspace-show-settings-mcp-servers = Открыть настройки: Серверы MCP
keybinding-desc-workspace-open-settings-file = Открыть файл настроек

# Overflow menu / external links
keybinding-desc-workspace-link-to-slack = Присоединиться к нашему сообществу Slack (открывает внешнюю ссылку)
keybinding-desc-workspace-link-to-user-docs = Показать документацию пользователя (открывает внешнюю ссылку)
keybinding-desc-workspace-send-feedback = Отправить отзыв (открывает внешнюю ссылку)
keybinding-desc-workspace-send-feedback-oz = Отправить отзыв через Oz
keybinding-desc-workspace-view-logs = Показать журналы Zap
keybinding-desc-workspace-link-to-privacy-policy = Показать политику конфиденциальности (открывает внешнюю ссылку)

# Input / terminal / project bindings (registered outside workspace/mod.rs)
keybinding-desc-input-edit-prompt = Редактировать промпт
keybinding-desc-terminal-attach-block-as-context = Прикрепить выбранный блок как контекст агента
keybinding-desc-terminal-attach-text-as-context = Прикрепить выбранный текст как контекст агента
keybinding-desc-terminal-attach-as-context-menu = Прикрепить выделенное как контекст агента
keybinding-desc-workspace-init-project = Инициализировать проект для Zap
keybinding-desc-workspace-add-current-folder = Добавить текущую папку как проект

# Workspace debug / crash / heap profile bindings
keybinding-desc-workspace-crash-macos = Вызвать сбой приложения (для тестирования локальной отчетности о сбоях)
keybinding-desc-workspace-crash-other = Вызвать сбой приложения (для тестирования локальной отчетности о сбоях)
keybinding-desc-workspace-log-review-comment-send-status = [Debug] Записать в журнал статус отправки комментария ревью для активной вкладки
keybinding-desc-workspace-panic = Вызвать панику (для тестирования локального журналирования паники)
keybinding-desc-workspace-open-view-tree-debugger = Открыть отладчик дерева представлений
keybinding-desc-workspace-view-first-time-user-experience = [Debug] Показать сценарий первого запуска
keybinding-desc-workspace-undismiss-aws-login-banner = [Debug] Показать скрытый баннер входа AWS
keybinding-desc-workspace-open-oz-launch-modal = [Debug] Открыть модальное окно запуска Oz
keybinding-desc-workspace-reset-oz-launch-modal-state = [Debug] Сбросить состояние модального окна запуска Oz
keybinding-desc-workspace-open-zap-launch-modal = [Debug] Открыть модальное окно запуска Zap
keybinding-desc-workspace-reset-zap-launch-modal-state = [Debug] Сбросить состояние модального окна запуска Zap
keybinding-desc-workspace-install-opencode-warp-plugin = [Debug] Установить плагин OpenCode Warp
keybinding-desc-workspace-use-local-opencode-warp-plugin = [Debug] Использовать локальный плагин OpenCode Warp (только для тестирования)
keybinding-desc-workspace-open-session-config-modal = [Debug] Открыть модальное окно конфигурации сессии
keybinding-desc-workspace-start-hoa-onboarding-flow = [Debug] Запустить сценарий Onboarding HOA
keybinding-desc-workspace-sample-process = Профилировать процесс
keybinding-desc-workspace-dump-heap-profile = Создать дамп профиля кучи (можно выполнить только один раз)

# Terminal input bindings
keybinding-desc-input-show-network-log = Показать сетевой журнал Zap
keybinding-desc-input-clear-screen = Очистить экран
keybinding-desc-input-toggle-classic-completions = (Экспериментально) Переключить классический режим автодополнения
keybinding-desc-input-command-search = Поиск команд
keybinding-desc-input-history-search = Поиск по истории
keybinding-desc-input-open-completions-menu = Открыть меню автодополнения
keybinding-desc-input-workflows = Workflows
keybinding-desc-input-open-ai-command-suggestions = Открыть AI-подсказки команд
keybinding-desc-input-new-agent-conversation = Новый разговор с агентом
keybinding-desc-input-trigger-auto-detection = Запустить автоопределение
keybinding-desc-input-clear-and-reset-ai-context-menu-query = Очистить и сбросить запрос в меню AI-контекста

# Terminal view bindings
keybinding-desc-terminal-alternate-paste = Альтернативная вставка в терминал
keybinding-desc-terminal-toggle-cli-agent-rich-input = Переключить расширенный ввод CLI-агента
keybinding-desc-terminal-warpify-subshell = Warpify подоболочки
keybinding-desc-terminal-warpify-ssh-session = Warpify SSH-сессии
keybinding-desc-terminal-accept-prompt-suggestion = Принять подсказку промпта
keybinding-desc-terminal-cancel-process-windows = Копировать текст или отменить активный процесс
keybinding-desc-terminal-cancel-process = Отменить активный процесс
keybinding-desc-terminal-focus-input = Перейти к полю ввода терминала
keybinding-desc-terminal-paste = Вставить
keybinding-desc-terminal-copy = Копировать
keybinding-desc-terminal-reinput-commands = Повторно ввести выбранные команды
keybinding-desc-terminal-reinput-commands-sudo = Повторно ввести выбранные команды от имени root
keybinding-desc-terminal-find = Поиск в терминале
keybinding-desc-terminal-select-bookmark-up = Выбрать ближайшую закладку выше
keybinding-desc-terminal-select-bookmark-down = Выбрать ближайшую закладку ниже
keybinding-desc-terminal-open-block-context-menu = Открыть контекстное меню блока
keybinding-desc-terminal-toggle-workflows-modal = Переключить модальное окно workflow
keybinding-desc-terminal-copy-git-branch = Копировать ветку git
keybinding-desc-terminal-clear-blocks = Очистить блоки
keybinding-desc-terminal-cursor-word-left = Переместить курсор на одно слово влево внутри выполняемой команды
keybinding-desc-terminal-cursor-word-right = Переместить курсор на одно слово вправо внутри выполняемой команды
keybinding-desc-terminal-cursor-home = Переместить курсор в начало внутри выполняемой команды
keybinding-desc-terminal-cursor-end = Переместить курсор в конец внутри выполняемой команды
keybinding-desc-terminal-delete-word-left = Удалить слово слева внутри выполняемой команды
keybinding-desc-terminal-delete-line-start = Удалить до начала строки внутри выполняемой команды
keybinding-desc-terminal-delete-line-end = Удалить до конца строки внутри выполняемой команды
keybinding-desc-terminal-backward-tabulation = Табуляция назад внутри выполняемой команды
keybinding-desc-terminal-select-previous-block = Выбрать предыдущий блок
keybinding-desc-terminal-select-next-block = Выбрать следующий блок
keybinding-desc-terminal-share-selected-block = Поделиться выбранным блоком
keybinding-desc-terminal-bookmark-selected-block = Добавить выбранный блок в закладки
keybinding-desc-terminal-find-within-selected-block = Поиск внутри выбранного блока
keybinding-desc-terminal-copy-command-and-output = Копировать команду и вывод
keybinding-desc-terminal-copy-command-output = Копировать вывод команды
keybinding-desc-terminal-copy-command = Копировать команду
keybinding-desc-terminal-scroll-up-one-line = Прокрутить вывод терминала на одну строку вверх
keybinding-desc-terminal-scroll-down-one-line = Прокрутить вывод терминала на одну строку вниз
keybinding-desc-terminal-scroll-up-one-page = Прокрутить вывод терминала на одну страницу вверх
keybinding-desc-terminal-scroll-down-one-page = Прокрутить вывод терминала на одну страницу вниз
keybinding-desc-terminal-scroll-to-top-of-block = Прокрутить к началу выбранного блока
keybinding-desc-terminal-scroll-to-bottom-of-block = Прокрутить к концу выбранного блока
keybinding-desc-terminal-select-all-blocks = Выбрать все блоки
keybinding-desc-terminal-expand-blocks-above = Расширить выделение на блоки сверху
keybinding-desc-terminal-expand-blocks-below = Расширить выделение на блоки снизу
keybinding-desc-terminal-insert-command-correction = Вставить исправление команды
keybinding-desc-terminal-setup-guide = Руководство по настройке
keybinding-desc-terminal-onboarding-warp-input-terminal = [Debug] Подсказка Onboarding: WarpInput - Терминал
keybinding-desc-terminal-onboarding-warp-input-project = [Debug] Подсказка Onboarding: WarpInput - Проект
keybinding-desc-terminal-onboarding-warp-input-no-project = [Debug] Подсказка Onboarding: WarpInput - Без проекта
keybinding-desc-terminal-onboarding-modality-project = [Debug] Подсказка Onboarding: Режим - Проект
keybinding-desc-terminal-onboarding-modality-no-project = [Debug] Подсказка Onboarding: Режим - Без проекта
keybinding-desc-terminal-onboarding-modality-terminal = [Debug] Подсказка Onboarding: Режим - Терминал
keybinding-desc-terminal-import-external-settings = Импортировать внешние настройки
keybinding-desc-terminal-share-current-session = Поделиться текущей сессией
keybinding-desc-terminal-stop-sharing-current-session = Прекратить общий доступ к текущей сессии
keybinding-desc-terminal-toggle-block-filter = Переключить фильтр блоков на выбранном или последнем блоке
keybinding-desc-terminal-toggle-sticky-command-header = Переключить закрепленный заголовок команды в активной панели
keybinding-desc-terminal-toggle-autoexecute-mode = Переключить режим автовыполнения
keybinding-desc-terminal-toggle-queue-next-prompt = Переключить очередь следующего промпта

# Pane group bindings
keybinding-desc-pane-group-close-current-session = Закрыть текущую сессию
keybinding-desc-pane-group-split-left = Разделить панель влево
keybinding-desc-pane-group-split-up = Разделить панель вверх
keybinding-desc-pane-group-split-down = Разделить панель вниз
keybinding-desc-pane-group-split-right = Разделить панель вправо
keybinding-desc-pane-group-switch-left = Переключиться на панель слева
keybinding-desc-pane-group-switch-right = Переключиться на панель справа
keybinding-desc-pane-group-switch-up = Переключиться на панель сверху
keybinding-desc-pane-group-switch-down = Переключиться на панель снизу
keybinding-desc-pane-group-resize-left = Изменить размер панели > Сдвинуть разделитель влево
keybinding-desc-pane-group-resize-right = Изменить размер панели > Сдвинуть разделитель вправо
keybinding-desc-pane-group-resize-up = Изменить размер панели > Сдвинуть разделитель вверх
keybinding-desc-pane-group-resize-down = Изменить размер панели > Сдвинуть разделитель вниз
keybinding-desc-pane-group-toggle-maximize = Развернуть активную панель

# Root view bindings
keybinding-desc-root-view-toggle-fullscreen = Переключить полноэкранный режим
keybinding-desc-root-view-enter-onboarding-state = [Debug] Войти в состояние Onboarding

# Workflow view bindings
keybinding-desc-workflow-view-save = Сохранить workflow
keybinding-desc-workflow-view-close = Закрыть

# Editor view binding desc (shared by editor/view/mod.rs, code/editor/view/actions.rs, notebooks/editor/view.rs)
keybinding-desc-editor-copy = Копировать
keybinding-desc-editor-cut = Вырезать
keybinding-desc-editor-paste = Вставить
keybinding-desc-editor-undo = Отменить
keybinding-desc-editor-redo = Повторить
keybinding-desc-editor-select-left-by-word = Выделить одно слово слева
keybinding-desc-editor-select-right-by-word = Выделить одно слово справа
keybinding-desc-editor-select-left = Выделить один символ слева
keybinding-desc-editor-select-right = Выделить один символ справа
keybinding-desc-editor-select-up = Выделить вверх
keybinding-desc-editor-select-down = Выделить вниз
keybinding-desc-editor-select-all = Выделить все
keybinding-desc-editor-select-to-line-start = Выделить до начала строки
keybinding-desc-editor-select-to-line-end = Выделить до конца строки
keybinding-desc-editor-select-to-line-start-cap = Выделить до начала строки
keybinding-desc-editor-select-to-line-end-cap = Выделить до конца строки
keybinding-desc-editor-clear-and-copy-lines = Копировать и очистить выбранные строки
keybinding-desc-editor-add-next-occurrence = Добавить следующее вхождение к выделению
keybinding-desc-editor-up = Переместить курсор вверх
keybinding-desc-editor-down = Переместить курсор вниз
keybinding-desc-editor-left = Переместить курсор влево
keybinding-desc-editor-right = Переместить курсор вправо
keybinding-desc-editor-move-to-line-start = Перейти к началу строки
keybinding-desc-editor-move-to-line-end = Перейти к концу строки
keybinding-desc-editor-move-to-line-start-short = Перейти к началу строки
keybinding-desc-editor-move-to-line-end-short = Перейти к концу строки
keybinding-desc-editor-home = Home
keybinding-desc-editor-end = End
keybinding-desc-editor-cmd-down = Переместить курсор в самый низ
keybinding-desc-editor-cmd-up = Переместить курсор в самый верх
keybinding-desc-editor-move-to-and-select-buffer-start = Выделить и перейти в самое начало
keybinding-desc-editor-move-to-and-select-buffer-end = Выделить и перейти в самый конец
keybinding-desc-editor-move-forward-one-word = Перейти на одно слово вперед
keybinding-desc-editor-move-backward-one-word = Перейти на одно слово назад
keybinding-desc-editor-move-forward-one-word-cap = Перейти на одно слово вперед
keybinding-desc-editor-move-backward-one-word-cap = Перейти на одно слово назад
keybinding-desc-editor-move-to-paragraph-start = Перейти к началу абзаца
keybinding-desc-editor-move-to-paragraph-end = Перейти к концу абзаца
keybinding-desc-editor-move-to-paragraph-start-short = Перейти к началу абзаца
keybinding-desc-editor-move-to-paragraph-end-short = Перейти к концу абзаца
keybinding-desc-editor-move-to-buffer-start = Перейти к началу буфера
keybinding-desc-editor-move-to-buffer-end = Перейти к концу буфера
keybinding-desc-editor-cursor-at-buffer-start = Курсор в начале буфера
keybinding-desc-editor-cursor-at-buffer-end = Курсор в конце буфера
keybinding-desc-editor-backspace = Удалить предыдущий символ
keybinding-desc-editor-cut-word-left = Вырезать слово слева
keybinding-desc-editor-cut-word-right = Вырезать слово справа
keybinding-desc-editor-delete-word-left = Удалить слово слева
keybinding-desc-editor-delete-word-right = Удалить слово справа
keybinding-desc-editor-cut-all-left = Вырезать все слева
keybinding-desc-editor-cut-all-right = Вырезать все справа
keybinding-desc-editor-delete-all-left = Удалить все слева
keybinding-desc-editor-delete-all-right = Удалить все справа
keybinding-desc-editor-delete = Удалить
keybinding-desc-editor-clear-lines = Очистить выбранные строки
keybinding-desc-editor-insert-newline = Вставить новую строку
keybinding-desc-editor-fold = Свернуть
keybinding-desc-editor-unfold = Развернуть
keybinding-desc-editor-fold-selected-ranges = Свернуть выбранные диапазоны
keybinding-desc-editor-insert-last-word-prev-cmd = Вставить последнее слово предыдущей команды
keybinding-desc-editor-move-backward-one-subword = Перейти на одну часть слова назад
keybinding-desc-editor-move-forward-one-subword = Перейти на одну часть слова вперед
keybinding-desc-editor-select-left-by-subword = Выделить одну часть слова слева
keybinding-desc-editor-select-right-by-subword = Выделить одну часть слова справа
keybinding-desc-editor-accept-autosuggestion = Принять автоподсказку
keybinding-desc-editor-inspect-command = Просмотреть команду
keybinding-desc-editor-clear-buffer = Очистить редактор команды
keybinding-desc-editor-add-cursor-above = Добавить курсор выше
keybinding-desc-editor-add-cursor-below = Добавить курсор ниже
keybinding-desc-editor-insert-nonexpanding-space = Вставить нерасширяющийся пробел
keybinding-desc-editor-vim-exit-insert-mode = Выйти из режима вставки Vim
keybinding-desc-editor-toggle-comment = Закомментировать или раскомментировать
keybinding-desc-editor-go-to-line = Перейти к строке
keybinding-desc-editor-find-in-code-editor = Поиск в редакторе кода

# Code editor (Code) binding desc
keybinding-desc-code-save-as = Сохранить файл как
keybinding-desc-code-close-all-tabs = Закрыть все вкладки
keybinding-desc-code-close-saved-tabs = Закрыть сохраненные вкладки

# Welcome view binding desc
keybinding-desc-welcome-terminal-session = Сессия терминала
keybinding-desc-welcome-add-repository = Добавить репозиторий

# AI assistant panel binding desc
keybinding-desc-ai-assistant-close = Закрыть Zap AI
keybinding-desc-ai-assistant-focus-terminal-input = Перевести фокус на ввод в терминале из Zap AI
keybinding-desc-ai-assistant-restart = Перезапустить Zap AI

# Code review binding desc
keybinding-desc-code-review-save-all = Сохранить все несохраненные файлы в Code Review
keybinding-desc-code-review-show-find = Показать панель поиска в Code Review

# Project buttons binding desc
keybinding-desc-project-buttons-open-repository = Открыть репозиторий
keybinding-desc-project-buttons-create-new-project = Создать новый проект

# Find view binding desc
keybinding-desc-find-next-occurrence = Найти следующее вхождение поискового запроса
keybinding-desc-find-prev-occurrence = Найти предыдущее вхождение поискового запроса

# Notebook file / notebook binding desc
keybinding-desc-notebook-focus-terminal-input-from-file = Перевести фокус на ввод в терминале из файла
keybinding-desc-notebook-reload-file = Перезагрузить файл
keybinding-desc-notebook-increase-font-size = Увеличить размер шрифта блокнота
keybinding-desc-notebook-decrease-font-size = Уменьшить размер шрифта блокнота
keybinding-desc-notebook-reset-font-size = Сбросить размер шрифта блокнота
keybinding-desc-notebook-focus-terminal-input = Перевести фокус на ввод в терминале из блокнота
keybinding-desc-notebook-fb-increase-font-size = Увеличить размер шрифта
keybinding-desc-notebook-fb-decrease-font-size = Уменьшить размер шрифта

# Notebook editor binding desc (extra to shared editor keys)
keybinding-desc-nbeditor-deselect-command = Снять выделение с команд shell
keybinding-desc-nbeditor-select-command = Выделить команду shell под курсором
keybinding-desc-nbeditor-select-previous-command = Выделить предыдущую команду
keybinding-desc-nbeditor-select-next-command = Выделить следующую команду
keybinding-desc-nbeditor-run-commands = Выполнить выбранные команды
keybinding-desc-nbeditor-toggle-debug = Переключить режим отладки форматированного текста
keybinding-desc-nbeditor-debug-copy-buffer = Копировать буфер форматированного текста
keybinding-desc-nbeditor-debug-copy-selection = Копировать выделение форматированного текста
keybinding-desc-nbeditor-log-state = Вывести состояние редактора в лог
keybinding-desc-nbeditor-edit-link = Создать или изменить ссылку
keybinding-desc-nbeditor-inline-code = Переключить оформление строчного кода
keybinding-desc-nbeditor-strikethrough = Переключить зачеркивание
keybinding-desc-nbeditor-underline = Переключить подчеркивание
keybinding-desc-nbeditor-find = Поиск в блокноте
keybinding-desc-nbeditor-next-find-match = Перейти к следующему совпадению
keybinding-desc-nbeditor-previous-find-match = Перейти к предыдущему совпадению
keybinding-desc-nbeditor-toggle-regex-find = Переключить поиск по регулярному выражению
keybinding-desc-nbeditor-toggle-case-sensitive-find = Переключить поиск с учетом регистра

# Pane group / undo close binding desc
keybinding-desc-get-started-terminal-session = Сессия терминала
keybinding-desc-undo-close-reopen-session = Повторно открыть закрытую сессию
keybinding-desc-right-panel-toggle-maximize-code-review = Развернуть или свернуть панель Code Review

# Workspace sync inputs binding desc
keybinding-desc-workspace-disable-sync-inputs = Прекратить синхронизацию панелей
keybinding-desc-workspace-toggle-sync-inputs-tab = Переключить синхронизацию всех панелей в текущей вкладке
keybinding-desc-workspace-toggle-sync-inputs-all-tabs = Переключить синхронизацию всех панелей во всех вкладках

# Workspace a11y / debug binding desc
keybinding-desc-workspace-a11y-concise = [a11y] Включить краткие объявления доступности
keybinding-desc-workspace-a11y-verbose = [a11y] Включить подробные объявления доступности
keybinding-desc-workspace-copy-access-token = Скопировать токен доступа в буфер обмена

# Env var collection binding desc
keybinding-desc-env-var-collection-close = Закрыть

# Auth / share modal binding desc
keybinding-desc-share-block-copy = Копировать
keybinding-desc-auth-paste-token = Вставить
keybinding-desc-conversation-details-copy = Копировать

# Terminal extras binding desc
keybinding-desc-terminal-show-history = Показать историю
keybinding-desc-terminal-ask-ai-selection = Спросить Zap AI о выделении
keybinding-desc-terminal-ask-ai-last-block = Спросить Zap AI о последнем блоке
keybinding-desc-terminal-ask-ai = Спросить Zap AI
keybinding-desc-terminal-load-agent-conversation = Загрузить беседу режима агента (по отладочной ссылке из буфера обмена)
keybinding-desc-terminal-toggle-session-recording = Переключить запись PTY для сессии

# Notebook editor extra
keybinding-desc-nbeditor-select-to-paragraph-start = Выделить до начала абзаца
keybinding-desc-nbeditor-select-to-paragraph-end = Выделить до конца абзаца

# Misc binding desc(收尾批次:常量/LazyLock/动态描述去硬编码)
keybinding-desc-save-file = Сохранить файл
keybinding-desc-new-agent-pane = Новая панель агента
keybinding-desc-edit-code-diff = Редактировать diff кода
keybinding-desc-edit-requested-command = Изменить запрошенную команду
keybinding-desc-set-input-mode-agent = Переключить режим ввода на режим агента
keybinding-desc-set-input-mode-terminal = Переключить режим ввода на режим терминала
keybinding-desc-toggle-hide-cli-responses = Переключить скрытие ответов CLI
keybinding-desc-slash-command = Slash-команда: { $name }
keybinding-desc-take-control-of-running-command = Перехватить управление выполняемой командой

# --- Terminal zero-state block (welcome chips) ---
terminal-zero-state-title = Новая сессия терминала
terminal-zero-state-start-agent = начать новую беседу с агентом
terminal-zero-state-cycle-history = листать прошлые команды и беседы
terminal-zero-state-open-code-review = открыть Code Review
terminal-zero-state-autodetect-prompts = автоматически определять промпты агента в сессиях терминала
terminal-zero-state-dismiss = Больше не показывать

# --- Rules page (ai/facts/view/rule.rs) ---
rules-description = Правила улучшают работу агента, предоставляя структурированные рекомендации, которые помогают поддерживать согласованность, соблюдать лучшие практики и адаптироваться к конкретным workflow, включая кодовые базы и более масштабные задачи.
rules-search-placeholder = Поиск правил
rules-name-placeholder = например, правила для Rust
rules-description-placeholder = например, никогда не используйте unwrap в Rust
rules-zero-state-global = Когда вы добавите правило, оно появится здесь.
rules-zero-state-project = Когда вы создадите файл правил WARP.md для проекта, он появится здесь.
rules-disabled-banner-prefix = Ваши правила отключены и не будут использоваться как контекст в сессиях. Вы можете {" "}
rules-disabled-banner-link = включить их снова
rules-disabled-banner-suffix = {" "}в любое время.
rules-tab-global = Глобальные
rules-tab-project = По проектам
rules-add-button = Добавить
rules-init-project-button = Инициализировать проект

# --- Agent view zero-state + message bar ---
agent-zero-state-title = Новая беседа с агентом Oz
agent-zero-state-description = Отправьте промпт ниже, чтобы начать новую беседу
agent-zero-state-description-with-location = Отправьте промпт ниже, чтобы начать новую беседу в `{ $location }`
agent-zero-state-recent-activity = НЕДАВНЯЯ АКТИВНОСТЬ
agent-zero-state-hide-hints-tooltip = Скрыть подсказки сочетаний клавиш (включить снова в настройках)
inline-agent-header-prompt-to-interact-command = Попросить агента взаимодействовать с `{ $command }`
inline-agent-header-prompt-to-interact-running-command = Попросить агента взаимодействовать с выполняемой командой
inline-agent-header-waiting-on-instructions = Агент ожидает инструкций
inline-agent-header-waiting-for-command = Агент ожидает завершения команды
inline-agent-header-agent-blocked = Агенту требуется ваше разрешение для продолжения
inline-agent-header-agent-in-control = Управляет агент
inline-agent-header-user-in-control = Управляет пользователь
agent-toolbar-edit-agent-toolbelt = Редактировать набор инструментов агента
agent-toolbar-edit-cli-agent-toolbelt = Редактировать набор инструментов CLI-агента
agent-toolbar-available-chips = Доступные чипы
agent-message-bar-get-figma-mcp = Получить Figma MCP
agent-message-bar-enable-figma-mcp = Включить Figma MCP
agent-message-bar-enabling = Включение…
child-agent-default-name = Агент
agent-zero-state-switch-model = сменить модель
agent-zero-state-go-back-to-terminal = вернуться в терминал
agent-message-bar-for-help = для справки
agent-message-bar-for-commands = для команд
agent-message-bar-open-conversation = открыть беседу
agent-message-bar-for-code-review = для Code Review
agent-message-bar-resume-conversation = чтобы возобновить беседу
agent-message-bar-hide-plan = чтобы скрыть план
agent-message-bar-view-plans = чтобы посмотреть планы
agent-message-bar-view-plan = чтобы посмотреть план
agent-message-bar-fork-continue = чтобы создать ответвление и продолжить
agent-message-bar-new-pane = {" "}новая панель
agent-message-bar-new-tab = {" "}новая вкладка
agent-message-bar-current-pane = {" "}текущая панель
agent-message-bar-hide-help = чтобы скрыть справку
agent-message-bar-autodetected-shell-command-prefix = автоматически определенная команда shell, {" "}
agent-message-bar-autodetected-shell-command = автоматически определенная команда shell
agent-message-bar-override = {" "}чтобы переопределить
agent-message-bar-exit-shell-mode = чтобы выйти из режима shell
agent-message-bar-again-stop-exit = еще раз, чтобы остановить и выйти
agent-message-bar-again-exit = еще раз, чтобы выйти
agent-message-bar-again-start-new-conversation = еще раз, чтобы начать новую беседу
agent-shortcuts-input-shell-command = ввод команды shell
agent-shortcuts-slash-commands = для slash-команд
agent-shortcuts-file-paths-context = для путей к файлам и прикрепления другого контекста
agent-shortcuts-open-code-review = открыть Code Review
agent-shortcuts-toggle-conversation-list = переключить список бесед
agent-shortcuts-search-continue-conversations = искать и продолжать беседы
agent-shortcuts-start-new-conversation = начать новую беседу
agent-shortcuts-toggle-auto-accept = переключить автопринятие
agent-shortcuts-pause-agent = приостановить агента
agent-error-will-resume-when-network-restored = Беседа возобновится, когда восстановится сетевое соединение…
agent-error-attempting-resume-conversation = Пытаемся возобновить беседу…

# --- ANCHOR-SUB-TOGGLE-PAIR (settings-toggle-pair) ---
toggle-setting-enable = Включить { $suffix }
toggle-setting-disable = Отключить { $suffix }

toggle-suffix-active-ai = активный AI
toggle-suffix-ai-input-autodetect-agent = автоопределение команд терминала в поле ввода агента
toggle-suffix-ai-input-autodetect-nld = определение естественного языка
toggle-suffix-nld-in-terminal = автоопределение промптов агента в поле ввода терминала
toggle-suffix-next-command = следующую команду
toggle-suffix-prompt-suggestions = подсказки промптов
toggle-suffix-code-suggestions = подсказки по коду
toggle-suffix-nl-autosuggestions = автоподсказки на естественном языке
toggle-suffix-voice-input = голосовой ввод
toggle-suffix-codebase-index = индекс кодовой базы
toggle-suffix-auto-indexing = автоиндексирование
toggle-suffix-compact-mode = компактный режим
toggle-suffix-themes-sync-os = темы: синхронизация с ОС
toggle-suffix-cursor-blink = мерцание курсора
toggle-suffix-jump-bottom-block = кнопку перехода к низу блока
toggle-suffix-block-dividers = разделители блоков
toggle-suffix-dim-inactive-panes = затемнение неактивных панелей
toggle-suffix-tab-indicators = индикаторы вкладок
toggle-suffix-focus-follows-mouse = фокус под курсором мыши
toggle-suffix-zen-mode = дзен-режим
toggle-suffix-vertical-tabs = вертикальное расположение вкладок
toggle-suffix-ligature-rendering = отображение лигатур
toggle-suffix-copy-on-select = копирование при выделении в терминале
toggle-suffix-linux-selection-clipboard = буфер обмена выделения Linux
toggle-suffix-autocomplete-symbols = автозакрытие кавычек, круглых и квадратных скобок
toggle-suffix-restore-session = восстановление окон, вкладок и панелей при запуске
toggle-suffix-left-option-meta = левую клавишу Option как Meta
toggle-suffix-left-alt-meta = левую клавишу Alt как Meta
toggle-suffix-right-option-meta = правую клавишу Option как Meta
toggle-suffix-right-alt-meta = правую клавишу Alt как Meta
toggle-suffix-scroll-reporting = отчеты о прокрутке
toggle-suffix-completions-while-typing = автодополнение во время ввода
toggle-suffix-command-corrections = исправление команд
toggle-suffix-error-underlining = подчеркивание ошибок
toggle-suffix-syntax-highlighting = подсветка синтаксиса
toggle-suffix-audible-bell = звуковой сигнал терминала
toggle-suffix-autosuggestions = автоподсказки
toggle-suffix-autosuggestion-keybinding-hint = подсказку сочетания клавиш для автоподсказок
toggle-suffix-ssh-wrapper = обертку Zap SSH
toggle-suffix-ssh-auto-discovery = автообнаружение хостов SSH
toggle-suffix-link-tooltip = показ подсказки при клике по ссылкам
toggle-suffix-quit-warning = окно предупреждения при выходе
toggle-suffix-alias-expansion = раскрытие псевдонимов
toggle-suffix-middle-click-paste = вставку средней кнопкой мыши
toggle-suffix-code-as-default-editor = Code как редактор по умолчанию
toggle-suffix-input-hint-text = текст подсказок ввода
toggle-suffix-vim-keybindings = редактирование команд сочетаниями клавиш Vim
toggle-suffix-vim-clipboard = безымянный регистр Vim как системный буфер обмена
toggle-suffix-vim-status-bar = строку состояния Vim
toggle-suffix-focus-reporting = отчеты о фокусе
toggle-suffix-smart-select = умное выделение
toggle-suffix-input-message-line = строку сообщений ввода терминала
toggle-suffix-slash-commands-terminal = slash-команды в режиме терминала
toggle-suffix-integrated-gpu = рендеринг на встроенном GPU (низкое энергопотребление)
toggle-suffix-wayland = Wayland для управления окнами
toggle-suffix-app-analytics = локальную диагностику
toggle-suffix-crash-reporting = отчеты о сбоях
toggle-suffix-secret-redaction = сокрытие секретов
toggle-suffix-recording-mode = режим записи
toggle-suffix-inband-generators = in-band генераторы для новых сессий
toggle-suffix-debug-network = отладку состояния сети
toggle-suffix-memory-stats = статистику памяти

# Set agent thinking display
agent-thinking-display-show-collapse = Настройка отображения размышлений агента: показывать и сворачивать
agent-thinking-display-always-show = Настройка отображения размышлений агента: показывать всегда
agent-thinking-display-never-show = Настройка отображения размышлений агента: никогда не показывать

# --- ANCHOR-SUB-EXTERNAL-EDITOR (settings-external-editor) ---
settings-external-editor-choose-default = Выберите редактор для открытия ссылок на файлы
settings-external-editor-choose-code-panels = Выберите редактор для открытия файлов из панели Code Review, обозревателя проектов и глобального поиска
settings-external-editor-choose-layout = Выберите расположение для открытия файлов в Zap
settings-external-editor-tabbed-header = Группировать файлы в одной панели редактора
settings-external-editor-tabbed-desc = Когда эта настройка включена, все файлы, открытые в одной вкладке, будут автоматически группироваться в одну панель редактора.
settings-external-editor-prefer-markdown = По умолчанию открывать файлы Markdown в собственном просмотрщике Markdown Zap
settings-external-editor-layout-split-pane = Разделенная панель
settings-external-editor-layout-new-tab = Новая вкладка
settings-external-editor-default-app = Приложение по умолчанию

# =============================================================================
# SECTION: context-menu (Owner: agent-context-menu)
# 鼠标右键弹出菜单。surface 前缀:menu-{block,input,ai-block,tab,pane,filetree,codeeditor}-*
# =============================================================================

# --- block 右键菜单(terminal/view.rs) ---
menu-block-copy = Копировать
menu-block-copy-url = Копировать URL
menu-block-copy-path = Копировать путь
menu-block-show-in-finder = Показать в Finder
menu-block-show-containing-folder = Показать родительскую папку
menu-block-open-in-warp = Открыть в Zap
menu-block-open-in-editor = Открыть в редакторе
menu-block-insert-into-input = Вставить в поле ввода
menu-block-copy-command = Копировать команду
menu-block-copy-commands = Копировать команды
menu-block-find-within-block = Поиск в блоке
menu-block-find-within-blocks = Поиск в блоках
menu-block-scroll-to-top-of-block = Прокрутить к началу блока
menu-block-scroll-to-top-of-blocks = Прокрутить к началу блоков
menu-block-scroll-to-bottom-of-block = Прокрутить к концу блока
menu-block-scroll-to-bottom-of-blocks = Прокрутить к концу блоков
menu-block-save-as-workflow = Сохранить как workflow
menu-block-ask-warp-ai = Спросить Zap AI
menu-block-copy-output = Копировать вывод
menu-block-copy-filtered-output = Копировать отфильтрованный вывод
menu-block-toggle-block-filter = Переключить фильтр блока
menu-block-toggle-bookmark = Добавить или удалить закладку
menu-block-copy-prompt = Копировать промпт
menu-block-copy-right-prompt = Копировать правый промпт
menu-block-copy-working-directory = Копировать рабочую директорию
menu-block-copy-git-branch = Копировать ветку git
menu-block-edit-prompt = Изменить промпт
menu-block-edit-cli-agent-toolbelt = Изменить инструментарий CLI-агента
menu-block-edit-agent-toolbelt = Изменить инструментарий агента
menu-block-split-pane-right = Разделить панель вправо
menu-block-split-pane-left = Разделить панель влево
menu-block-split-pane-down = Разделить панель вниз
menu-block-split-pane-up = Разделить панель вверх
menu-block-close-pane = Закрыть панель

# --- input 右键菜单(terminal/view.rs) ---
menu-input-cut = Вырезать
menu-input-copy = Копировать
menu-input-paste = Вставить
menu-input-select-all = Выбрать все
menu-input-command-search = Поиск команд
menu-input-ai-command-search = AI-поиск команд
menu-input-ask-warp-ai = Спросить Zap AI
menu-input-save-as-workflow = Сохранить как workflow
menu-input-hide-hint-text = Скрыть текст подсказки ввода
menu-input-show-hint-text = Показать текст подсказки ввода

# --- AI block overflow 菜单(terminal/view.rs) ---
menu-ai-block-copy = Копировать
menu-ai-block-copy-prompt = Копировать промпт
menu-ai-block-copy-output-as-markdown = Копировать вывод как Markdown
menu-ai-block-copy-url = Копировать URL
menu-ai-block-copy-path = Копировать путь
menu-ai-block-copy-command = Копировать команду
menu-ai-block-copy-git-branch = Копировать ветку git
menu-ai-block-save-as-prompt = Сохранить как промпт
menu-ai-block-copy-conversation-text = Копировать текст беседы
menu-ai-block-fork-from-here = Ответвить отсюда
menu-ai-block-rewind-to-before-here = Откатить к состоянию до этой точки
menu-ai-block-fork-from-last-query = Ответвить от последнего запроса
menu-ai-block-fork-from-query = Ответвить от «{ $query }»

# --- tab 右键菜单(tab.rs) ---
menu-tab-stop-sharing = Остановить общий доступ
menu-tab-stop-sharing-all = Остановить весь общий доступ
menu-tab-copy-link = Копировать ссылку
menu-tab-rename = Переименовать вкладку
menu-tab-reset-name = Сбросить имя вкладки
menu-tab-move-down = Переместить вкладку вниз
menu-tab-move-right = Переместить вкладку вправо
menu-tab-move-up = Переместить вкладку вверх
menu-tab-move-left = Переместить вкладку влево
menu-tab-close = Закрыть вкладку
menu-tab-close-other = Закрыть другие вкладки
menu-tab-close-below = Закрыть вкладки ниже
menu-tab-close-right = Закрыть вкладки справа
menu-tab-save-as-new-config = Сохранить как новую конфигурацию
menu-tab-default-no-color = По умолчанию (без цвета)

# --- pane header 溢出菜单(terminal/view/pane_impl.rs) ---
menu-pane-copy-link = Копировать ссылку
menu-pane-stop-sharing-session = Остановить трансляцию сессии
menu-pane-open-on-desktop = Открыть на рабочем столе

# --- 文件树右键菜单(code/file_tree/view.rs) ---
menu-filetree-open-in-new-pane = Открыть в новой панели
menu-filetree-open-in-new-tab = Открыть в новой вкладке
menu-filetree-open-file = Открыть файл
menu-filetree-new-file = Новый файл
menu-filetree-cd-to-directory = cd в директорию
menu-filetree-reveal-finder = Показать в Finder
menu-filetree-reveal-explorer = Показать в Проводнике
menu-filetree-reveal-file-manager = Показать в файловом менеджере
menu-filetree-rename = Переименовать
menu-filetree-delete = Удалить
menu-filetree-attach-as-context = Прикрепить как контекст
menu-filetree-copy-path = Копировать путь
menu-filetree-copy-relative-path = Копировать относительный путь

# --- 代码编辑器右键菜单(code/local_code_editor.rs) ---
menu-codeeditor-go-to-definition = Перейти к определению
menu-codeeditor-find-references = Найти ссылки

# --- 共享标签:附加为 agent 上下文(blocklist/view_util.rs) ---
menu-attach-as-agent-context = Прикрепить как контекст агента

# --- ANCHOR-SUB-SLASH-COMMANDS (agent-slash-commands) ---
# Slash command palette descriptions and argument hints
# (app/src/search/slash_command_menu/static_commands/commands.rs)
slash-cmd-agent-desc = Начать новую беседу
slash-cmd-add-mcp-desc = Добавить новый сервер MCP
slash-cmd-pr-comments-desc = Загрузить комментарии ревью из GitHub PR
slash-cmd-create-environment-desc = Создать окружение Oz (образ Docker + репозитории) с помощью пошаговой настройки
slash-cmd-create-environment-hint = <необязательные пути к репозиториям или URL GitHub>
slash-cmd-docker-sandbox-desc = Создать новую сессию терминала в песочнице Docker
slash-cmd-create-new-project-desc = Oz пошагово поможет вам создать новый проект разработки
slash-cmd-create-new-project-hint = <опишите, что вы хотите создать>
slash-cmd-open-skill-desc = Открыть markdown-файл навыка во встроенном редакторе Zap
slash-cmd-skills-desc = Вызвать навык
slash-cmd-add-prompt-desc = Добавить новый промпт агента
slash-cmd-add-rule-desc = Добавить новое глобальное правило для агента
slash-cmd-open-file-desc = Открыть файл в редакторе кода Zap
slash-cmd-open-file-hint = <path/to/file[:line[:col]]> или "@" для поиска
slash-cmd-rename-tab-desc = Переименовать текущую вкладку
slash-cmd-rename-tab-hint = <имя вкладки>
slash-cmd-fork-desc = Ответвить текущую беседу в новую панель или новую вкладку
slash-cmd-fork-hint = <необязательный промпт для отправки в ответвленной беседе>
slash-cmd-open-code-review-desc = Открыть ревью кода
slash-cmd-init-desc = Создать или обновить файл AGENTS.md
slash-cmd-open-project-rules-desc = Открыть файл правил проекта (AGENTS.md)
slash-cmd-open-mcp-servers-desc = Открыть серверы MCP
slash-cmd-open-settings-file-desc = Открыть файл настроек (TOML)
slash-cmd-changelog-desc = Открыть последний список изменений
slash-cmd-open-repo-desc = Переключиться на другой проиндексированный репозиторий
slash-cmd-open-rules-desc = Просмотреть все ваши глобальные правила и правила проекта
slash-cmd-new-desc = Начать новую беседу (псевдоним для /agent)
slash-cmd-model-desc = Переключить базовую модель агента
slash-cmd-profile-desc = Переключить активный профиль исполнения
slash-cmd-plan-desc = Попросить агента провести исследование и составить план задачи
slash-cmd-plan-hint = <опишите вашу задачу>
slash-cmd-compact-desc = Освободить контекст, сократив историю беседы
slash-cmd-compact-hint = <необязательные пользовательские инструкции для сжатия>
slash-cmd-compact-and-desc = Сжать беседу, а затем отправить следующий промпт
slash-cmd-compact-and-hint = <промпт для отправки после сжатия>
slash-cmd-queue-desc = Поставить промпт в очередь для отправки после завершения ответа агента
slash-cmd-queue-hint = <промпт для отправки, когда агент закончит>
slash-cmd-fork-and-compact-desc = Ответвить текущую беседу и сжать ее в ответвленной копии
slash-cmd-fork-and-compact-hint = <необязательный промпт для отправки после сжатия>
slash-cmd-fork-from-desc = Ответвить беседу от конкретного запроса
slash-cmd-remote-control-desc = Запустить удаленное управление этой сессией
slash-cmd-conversations-desc = Открыть историю бесед
slash-cmd-prompts-desc = Найти сохраненные промпты
slash-cmd-rewind-desc = Откатить беседу к более ранней точке
slash-cmd-export-to-clipboard-desc = Экспортировать текущую беседу в буфер обмена в формате markdown
slash-cmd-export-to-file-desc = Экспортировать текущую беседу в markdown-файл
slash-cmd-export-to-file-hint = <необязательное имя файла>

# --- ANCHOR-SUB-PROMPT-TIPS ---
# Prompt editor modal (app/src/prompt/editor_modal.rs)
prompt-editor-title = Изменить промпт
prompt-editor-warp-prompt-section = Промпт терминала Zap
prompt-editor-shell-prompt-section = Приглашение shell (PS1)
prompt-editor-restore-default = Восстановить по умолчанию
prompt-editor-same-line-prompt = Приглашение на той же строке
prompt-editor-separator = Разделитель
prompt-editor-cancel = Отмена
prompt-editor-save-changes = Сохранить изменения

# Welcome tips (app/src/tips/tip_view.rs)
welcome-tips-command-palette-title = Палитра команд
welcome-tips-command-palette-description = Легко открывайте все возможности Zap, не отрывая рук от клавиатуры.
welcome-tips-split-pane-title = Разделение панелей
welcome-tips-split-pane-description = Разделяйте вкладки на несколько панелей, чтобы собрать идеальную раскладку.
welcome-tips-history-search-title = Поиск по истории
welcome-tips-history-search-description = Находите, изменяйте и повторно запускайте ранее выполненные команды.
welcome-tips-ai-command-search-title = AI-поиск команд
welcome-tips-ai-command-search-description = Создавайте команды shell на естественном языке.
welcome-tips-theme-picker-title = Выбор темы
welcome-tips-theme-picker-description = Настройте Zap под себя, выбрав встроенную тему. Или создайте свою.
welcome-tips-shortcut-label = Сочетание клавиш
welcome-tips-skip = Пропустить советы приветствия
welcome-tips-complete-title = Готово!
welcome-tips-complete-description = Отличная работа — вы прошли все советы приветствия!
welcome-tips-close = Закрыть советы приветствия

# --- ANCHOR-SUB-SMALL-DIALOGS ---
# Rewind confirmation dialog (app/src/workspace/rewind_confirmation_dialog.rs)
rewind-dialog-title = Откат
rewind-dialog-body = Вы уверены, что хотите выполнить откат? Это вернет ваш код и беседу к состоянию до этого момента и отменит команды, которые агент сейчас выполняет. Копия исходной беседы будет сохранена в истории бесед.
rewind-dialog-info = Откат не затрагивает файлы, измененные вручную или через команды shell.
rewind-dialog-cancel = Отмена
rewind-dialog-confirm = Откатить

# --- ANCHOR-SUB-SEARCH-PALETTES ---
# Search palettes (app/src/search/command_palette/view.rs, app/src/search/welcome_palette/view.rs)
command-palette-search-placeholder = Поиск команды
command-palette-no-results = Ничего не найдено
command-palette-toast-cannot-switch-conversations = Невозможно переключить беседу, пока агент следит за командой.
command-palette-toast-cannot-start-new-conversation = Невозможно начать новую беседу, пока агент следит за командой.
command-palette-zero-state-recent = Недавние
command-palette-zero-state-suggested = Рекомендуемые
welcome-palette-search-placeholder = Код, сборка или поиск чего угодно…
welcome-palette-no-results = Ничего не найдено
search-filter-placeholder-history = Поиск по истории
search-filter-placeholder-workflows = Поиск workflows
search-filter-placeholder-agent-mode-workflows = Поиск промптов
search-filter-placeholder-notebooks = Поиск блокнотов
search-filter-placeholder-plans = Поиск планов
search-filter-placeholder-natural-language = напр. заменить строку в файле
search-filter-placeholder-actions = Поиск действий
search-filter-placeholder-sessions = Поиск сессий
search-filter-placeholder-conversations = Поиск бесед
search-filter-placeholder-historical-conversations = Поиск бесед из истории
search-filter-placeholder-launch-configurations = Поиск конфигураций запуска
search-filter-placeholder-drive = Поиск объектов в Drive
search-filter-placeholder-environment-variables = Поиск переменных окружения
search-filter-placeholder-prompt-history = Поиск по истории промптов
search-filter-placeholder-files = Поиск файлов
search-filter-placeholder-commands = Поиск команд
search-filter-placeholder-blocks = Поиск блоков
search-filter-placeholder-code = Поиск символов кода
search-filter-placeholder-rules = Поиск правил AI
search-filter-placeholder-repos = Поиск репозиториев кода
search-filter-placeholder-diff-sets = Поиск наборов diff
search-filter-placeholder-static-slash-commands = Поиск статических slash-команд
search-filter-placeholder-skills = Поиск навыков
search-filter-placeholder-base-models = Поиск базовых моделей
search-filter-placeholder-full-terminal-use-models = Поиск моделей полного управления терминалом
search-filter-placeholder-current-directory-conversations = Поиск бесед в текущей директории
search-filter-display-history = история
search-filter-display-workflows = workflows
search-filter-display-agent-mode-workflows = промпты
search-filter-display-notebooks = блокноты
search-filter-display-plans = планы
search-filter-display-natural-language = предложения команд AI
search-filter-display-actions = действия
search-filter-display-sessions = сессии
search-filter-display-conversations = беседы
search-filter-display-launch-configurations = конфигурации запуска
search-filter-display-drive = Zap Drive
search-filter-display-environment-variables = переменные окружения
search-filter-display-prompt-history = история промптов
search-filter-display-files = файлы
search-filter-display-commands = команды
search-filter-display-blocks = блоки
search-filter-display-code = код
search-filter-display-rules = правила
search-filter-display-repos = репозитории
search-filter-display-diff-sets = наборы diff
search-filter-display-static-slash-commands = slash-команды
search-filter-display-historical-conversations = беседы из истории
search-filter-display-skills = навыки
search-filter-display-base-models = базовые модели
search-filter-display-full-terminal-use-models = модели полного управления терминалом
search-filter-display-current-directory-conversations = беседы текущей директории
search-results-menu-no-results = Ничего не найдено
search-results-menu-prompts-title = Промпты
ai-context-diffset-uncommitted-changes = Незакоммиченные изменения
ai-context-diffset-changes-vs-main-branch = Изменения относительно ветки main
ai-context-diffset-changes-vs-branch = Изменения относительно ветки { $branch }
ai-context-diffset-uncommitted-changes-description = Все незакоммиченные изменения в рабочей директории
ai-context-diffset-changes-vs-main-branch-description = Все изменения по сравнению с веткой main
ai-context-diffset-changes-vs-branch-description = Все изменения по сравнению с { $branch }
ai-context-code-search-failed = Не удалось выполнить поиск по коду
ai-context-files-directory-accessibility-label = Директория: { $path }
ai-context-files-file-accessibility-label = Файл: { $path }
ai-context-blocks-just-now = Только что
ai-context-blocks-minutes-ago = { $count ->
        [one] { $count } минуту назад
        [few] { $count } минуты назад
        [many] { $count } минут назад
       *[other] { $count } минуты назад
    }
ai-context-blocks-hours-ago = { $count ->
        [one] { $count } час назад
        [few] { $count } часа назад
        [many] { $count } часов назад
       *[other] { $count } часа назад
    }
ai-context-blocks-days-ago = { $count ->
        [one] { $count } день назад
        [few] { $count } дня назад
        [many] { $count } дней назад
       *[other] { $count } дня назад
    }
ai-context-blocks-no-output = Нет вывода
ai-context-blocks-accessibility-label = Блок: { $command }

# --- ANCHOR-SUB-DRIVE-NAMING-IMPORT ---
# Drive naming dialog (app/src/drive/cloud_object_naming_dialog.rs)
drive-naming-notebook-name = Имя блокнота
drive-naming-folder-name = Имя папки
drive-naming-collection-name = Имя коллекции
drive-naming-create = Создать
drive-naming-cancel = Отмена
drive-naming-rename = Переименовать

# Drive import modal (app/src/drive/import/modal.rs, app/src/drive/import/modal_body.rs)
drive-import-title = Импорт
drive-import-close = Закрыть
drive-import-cancel = Отмена
drive-import-preparing = Подготовка…
drive-import-choose-files = Выбрать файлы…
drive-import-learn-file-support = Узнать больше о поддержке файлов и форматировании
drive-import-file-upload-error = Не удалось загрузить файл на сервер
drive-import-folder-upload-error = Не удалось загрузить папку на сервер

# Drive main panel and workflow editor (app/src/drive/index.rs, app/src/drive/workflows/*)
drive-title = Drive
drive-environment-variables = Переменные окружения
drive-folder = Папка
drive-notebook = Блокнот
drive-workflow = Workflow
drive-prompt = Промпт
drive-import = Импорт
drive-remove = Удалить
drive-new-folder = Новая папка
drive-new-notebook = Новый блокнот
drive-new-workflow = Новый workflow
drive-new-prompt = Новый промпт
drive-new-environment-variables = Новые переменные окружения
drive-offline-banner = Вы офлайн. Некоторые файлы будут только для чтения.
drive-sort-by = Сортировать по
drive-retry-sync = Повторить синхронизацию
drive-empty-trash = Очистить корзину
drive-trash-section-title = КОРЗИНА
drive-trash-title = Корзина
drive-trash-deletion-warning = Элементы в корзине будут удалены навсегда через 30 дней.
drive-team-space-zero-state = Командные пространства недоступны в локальных сборках. Управляйте workflow и блокнотами в разделе «Личное».
drive-sign-up-storage-limit = На этом устройстве действуют лимиты локального хранилища.
drive-local-storage-limit-description = На этом устройстве действуют лимиты локального хранилища. Удалите неиспользуемые элементы, чтобы освободить место для новых объектов Zap Drive.
drive-sign-up = Управлять локально
drive-copy-link = Копировать ссылку
drive-collapse-all = Свернуть все
drive-revert-to-server = Вернуть версию с сервера
drive-attach-to-active-session = Прикрепить к активной сессии
drive-copy-prompt = Копировать промпт
drive-copy-workflow-text = Копировать текст workflow
drive-copy-id = Копировать id
drive-copy-variables = Копировать переменные
drive-load-in-subshell = Загрузить в subshell
drive-delete-forever = Удалить навсегда
drive-rename = Переименовать
drive-retry = Повторить
drive-move-to-space = Переместить в { $space }
drive-open-on-desktop = Открыть в Desktop
drive-duplicate = Дублировать
drive-export = Экспортировать
drive-trash-menu = Корзина
drive-open = Открыть
drive-edit = Редактировать
drive-restore = Восстановить
drive-compare-plans = Сравнить тарифы
drive-manage-billing = Управлять оплатой
drive-object-type-notebook-plural = блокнотов
drive-object-type-workflow-plural = workflow
drive-object-type-folder-plural = папок
drive-object-type-env-var-collection-plural = коллекций переменных окружения
drive-object-type-object-plural = объектов
drive-object-type-notebooks = Блокноты
drive-object-type-workflows = Workflows
drive-object-type-environment-variables = Переменные окружения
drive-object-type-folders = Папки
drive-object-type-agent-workflows = Workflow агентов
drive-object-type-ai-fact = AI-факт
drive-object-type-rules = Правила
drive-object-type-mcp-server = MCP-сервер
drive-object-type-mcp-servers = MCP-серверы
drive-shared-object-limit-hit-banner-prefix = Вы достигли локального лимита { $object_type }.
drive-shared-object-limit-hit-banner = Вы достигли локального лимита { $object_type }.
drive-payment-issue-banner-prefix = Общие объекты ограничены из-за проблемы с оплатой подписки.
drive-payment-issue-banner-admin = Общие объекты ограничены из-за проблемы с оплатой подписки. Обновите платежные данные, чтобы восстановить доступ.
drive-payment-issue-banner-admin-enterprise = Общие объекты ограничены из-за проблемы с оплатой подписки. Напишите на support@warp.dev, чтобы восстановить доступ.
drive-payment-issue-banner-nonadmin = Общие объекты ограничены из-за проблемы с оплатой подписки. Обратитесь к администратору команды, чтобы восстановить доступ.
drive-empty-trash-title = Вы уверены, что хотите очистить корзину?
drive-empty-trash-body = Это действие нельзя отменить.
drive-empty-trash-confirm = Да, очистить корзину
drive-empty-trash-cancel = Отмена
workflow-title-placeholder = Безымянный workflow
workflow-description-placeholder = Добавить описание
workflow-title-input-placeholder = Добавить название
workflow-description-input-placeholder = Добавить описание
workflow-new-argument = Новый аргумент
workflow-arguments-label = Аргументы
workflow-argument-description-placeholder = Описание
workflow-argument-value-placeholder = Значение (необязательно)
workflow-default-value-placeholder = Значение по умолчанию (необязательно)
workflow-agent-mode-query-placeholder = Введите промпт здесь… (напр., «Создай функцию для сортировки массива объектов по дате» или «Помоги отладить этот React-компонент»).
workflow-save = Сохранить workflow
workflow-unsaved-changes = У вас есть несохраненные изменения.
workflow-keep-editing = Продолжить редактирование
workflow-discard-changes = Отменить изменения
workflow-ai-assist-autofill = Автозаполнение
workflow-ai-assist-loading = Загрузка
workflow-ai-assist-tooltip = Сгенерировать название, описания или параметры с помощью Zap AI
workflow-tooltip-restore-from-trash = Восстановить workflow из корзины
workflow-ai-assist-error-byop-required = Автозаполнение требует модель BYOP. Настройте провайдер и модель в Настройки → AI.
workflow-ai-assist-error-bad-command = Не удалось сгенерировать метаданные. Попробуйте еще раз с другой командой.
workflow-ai-assist-error-generic = Что-то пошло не так. Попробуйте еще раз.
workflow-ai-assist-error-rate-limited = Похоже, у вас закончились AI-кредиты. Попробуйте позже.
workflow-enum-new = Новый
workflow-alias-name-placeholder = имя алиаса
workflow-add-argument-tooltip = Добавить аргумент workflow

# Workspace panels (app/src/workspace/view/*)
workspace-conversation-list-search = Поиск
workspace-conversation-list-active = АКТИВНЫЕ
workspace-conversation-list-past = ПРОШЛЫЕ
workspace-conversation-list-view-all = Показать все
workspace-conversation-list-show-less = Показать меньше
workspace-conversation-list-empty-title = Разговоров пока нет
workspace-conversation-list-empty-description = Здесь появятся ваши активные и прошлые разговоры с локальными и фоновыми агентами.
workspace-conversation-list-new-conversation = Новый разговор
conversation-untitled = Безымянный разговор
conversation-deleted = Удаленный разговор
workspace-conversation-list-no-matching = Нет подходящих разговоров
workspace-conversation-list-delete = Удалить
workspace-conversation-list-delete-in-progress-error = Нельзя удалить разговор, пока он выполняется.
workspace-conversation-list-delete-ambient-tooltip = Разговоры фоновых агентов нельзя удалить
workspace-conversation-list-fork-new-pane = Форкнуть в новую панель
workspace-conversation-list-fork-new-tab = Форкнуть в новую вкладку
workspace-conversation-list-fallback-title = Разговор
command-palette-conversations-active-pane = Разговоры активной панели
command-palette-conversations-other-active = Другие активные разговоры
command-palette-conversations-past = Прошлые разговоры
command-palette-conversations-fork-current = Форкнуть текущий разговор
command-palette-conversations-fork-current-with-title = Форкнуть текущий разговор ({ $title })
command-palette-conversations-a11y-navigate = Нажмите Enter, чтобы перейти к разговору
command-palette-conversations-a11y-fork = Нажмите Enter, чтобы форкнуть текущий разговор в новый.
command-palette-conversations-a11y-new = Нажмите Enter, чтобы создать новый разговор.
workspace-left-panel-project-explorer = Проводник проекта
project-explorer-unavailable-title = Проводник проекта недоступен
project-explorer-unavailable-disabled-description = Проводник проекта требует доступ к локальному рабочему пространству. Откройте новую сессию или перейдите к активной сессии, чтобы просмотреть.
project-explorer-unavailable-remote-description = Проводник проекта требует доступ к локальному рабочему пространству, что не поддерживается в удаленных сессиях.
project-explorer-unavailable-wsl-description = Проводник проекта сейчас не работает в WSL.
workspace-left-panel-global-search = Глобальный поиск
workspace-left-panel-warp-drive = Zap Drive
workspace-left-panel-agent-conversations = Разговоры агента
workspace-left-panel-ssh-manager = Менеджер SSH
workspace-left-panel-server-file-browser = Файлы сервера
workspace-left-panel-skill-manager = Менеджер навыков
skill-manager-search-placeholder = Поиск навыков
skill-manager-filter-all = Все
skill-manager-filter-provider = Источник
skill-manager-meta-default = По умолчанию
skill-manager-meta-duplicate = Дубликат
skill-manager-empty = Нет навыков, подходящих под текущие фильтры.
skill-manager-preview-empty = Выберите навык, чтобы просмотреть SKILL.md.
workspace-left-panel-ssh-manager-placeholder = Менеджер SSH — скоро
workspace-left-panel-ssh-manager-detail-empty = Выберите сервер, чтобы увидеть его данные.
workspace-left-panel-ssh-manager-detail-host = Хост
workspace-left-panel-ssh-manager-detail-port = Порт
workspace-left-panel-ssh-manager-detail-user = Пользователь
workspace-left-panel-ssh-manager-detail-auth = Авторизация
workspace-left-panel-ssh-manager-detail-key-path = Путь к ключу
workspace-left-panel-ssh-manager-auth-password = Пароль
workspace-left-panel-ssh-manager-auth-key = Закрытый ключ
workspace-left-panel-ssh-manager-auth-onekey = OneKey
workspace-left-panel-ssh-manager-onekey-credential = Учетные данные
workspace-left-panel-ssh-manager-onekey-new = Новые учетные данные
workspace-left-panel-ssh-manager-onekey-label = Имя учетных данных
workspace-left-panel-ssh-manager-onekey-user = Пользователь учетных данных
workspace-left-panel-ssh-manager-onekey-password = Пароль учетных данных
workspace-left-panel-ssh-manager-onekey-password-required = Новым учетным данным OneKey нужен пароль.
workspace-left-panel-ssh-manager-onekey-save-before-connect = Сохраните учетные данные OneKey перед подключением.
workspace-left-panel-ssh-manager-onekey-select = Выбрать учетные данные
workspace-left-panel-ssh-manager-onekey-select-required = Выберите учетные данные OneKey.
workspace-left-panel-ssh-manager-onekey-manage = Управлять OneKey
workspace-left-panel-ssh-manager-onekey-manager-title = Менеджер OneKey
workspace-left-panel-ssh-manager-onekey-add = Добавить
workspace-left-panel-ssh-manager-onekey-delete = Удалить
workspace-left-panel-ssh-manager-onekey-type = Тип
workspace-left-panel-ssh-manager-onekey-type-password = Пароль
workspace-left-panel-ssh-manager-onekey-type-key = Закрытый ключ
workspace-left-panel-ssh-manager-onekey-key-path = Путь к ключу
workspace-left-panel-ssh-manager-onekey-key-path-required = Для учетных данных с закрытым ключом нужен путь к ключу.
workspace-left-panel-ssh-manager-onekey-secret = Пароль
workspace-left-panel-ssh-manager-onekey-save = Сохранить
workspace-left-panel-ssh-manager-onekey-label-required = Имя учетных данных не может быть пустым.
workspace-left-panel-ssh-manager-menu-new-folder = Новая папка
workspace-left-panel-ssh-manager-menu-new-server = Новый SSH-сервер
workspace-left-panel-ssh-manager-menu-edit = Редактировать
workspace-left-panel-ssh-manager-menu-connect = Подключить
workspace-left-panel-ssh-manager-menu-sftp = Файловый менеджер
workspace-left-panel-ssh-manager-menu-clone = Клонировать
workspace-left-panel-ssh-manager-menu-delete = Удалить
workspace-left-panel-ssh-manager-pane-hint = Редактирование полей и «Подключить» появятся в следующей итерации. Сейчас эта панель показывает сохраненную конфигурацию; меняйте ее через хранилище SQLite или будущий редактор.
workspace-left-panel-ssh-manager-pane-folder-body = Папка. Выберите сервер внутри этой папки, чтобы увидеть его данные, или щелкните папку правой кнопкой для создания / удаления.
workspace-left-panel-ssh-manager-server-missing = Сервер не найден. Возможно, он был удален в другом окне.
workspace-left-panel-ssh-manager-field-name = Имя
workspace-left-panel-ssh-manager-field-group = Группа
workspace-left-panel-ssh-manager-group-root = Корень
workspace-left-panel-ssh-manager-passphrase = Парольная фраза
workspace-left-panel-ssh-manager-save = Сохранить
workspace-left-panel-ssh-manager-status-saved = Сохранено.
workspace-left-panel-ssh-manager-error-name-required = Имя не может быть пустым.
workspace-left-panel-ssh-manager-error-port-invalid = Порт должен быть числом от 1 до 65535.
workspace-left-panel-ssh-manager-error-host-required = Хост не может быть пустым.
workspace-left-panel-ssh-manager-connect = Подключить
workspace-left-panel-ssh-manager-test = Проверить
workspace-left-panel-ssh-manager-testing = Проверка…
workspace-left-panel-ssh-manager-status-online = Онлайн
workspace-left-panel-ssh-manager-status-offline = Офлайн
workspace-left-panel-ssh-manager-status-unknown = Неизвестно
search-filter-placeholder-ssh-servers = Поиск SSH-серверов…
search-filter-display-ssh-servers = SSH-серверы
workspace-left-panel-ssh-manager-menu-rename = Переименовать
workspace-left-panel-ssh-manager-tree-empty = SSH-серверов пока нет. Нажмите 📁, чтобы добавить папку, + — чтобы добавить сервер.
workspace-left-panel-ssh-manager-root-password = Пароль root
workspace-left-panel-ssh-manager-root-password-placeholder = Пароль для перехода в root
workspace-left-panel-ssh-manager-startup-command = Команда запуска
workspace-left-panel-ssh-manager-startup-command-placeholder = Команда, выполняемая после подключения
workspace-left-panel-ssh-manager-notes = Заметки
workspace-left-panel-ssh-manager-notes-placeholder = Заметки
workspace-left-panel-ssh-manager-candidates-header = Из { $path }
workspace-left-panel-ssh-manager-candidates-empty = В { $path } нет хостов для импорта
workspace-left-panel-ssh-manager-candidates-not-found = SSH-конфиг не найден в { $path }
workspace-left-panel-ssh-manager-candidates-error = Не удалось прочитать SSH-конфиг в { $path }: { $error }
workspace-left-panel-ssh-manager-candidates-add = Добавить в менеджер SSH
workspace-left-panel-ssh-manager-candidates-added = Добавлено
workspace-left-panel-ssh-manager-candidates-refresh = Обновить из ~/.ssh/config
terminal-su-root-password-confirm = Автозаполнить пароль root
terminal-su-root-password-confirm-subtitle = Нажмите, чтобы подтвердить и подставить сохраненный пароль root
terminal-su-root-password-cancel = Отмена
server-file-browser-path-placeholder = Удаленный путь
server-file-browser-empty = Подключитесь к SSH-сессии, чтобы просматривать файлы сервера.
server-file-browser-no-session = Нет подключенной сессии удаленного сервера.
server-file-browser-connection-lost = Соединение с удаленным сервером потеряно. Переподключите SSH-сессию, затем обновите или заново откройте эту панель.
server-file-browser-loading = Загрузка…
server-file-browser-empty-directory = Этот каталог пуст.
server-file-browser-empty-response = Удаленный сервер вернул пустой ответ.
server-file-browser-unsupported-path = Этот тип удаленного пути пока не поддерживается.
server-file-browser-copied-path = Путь скопирован.
server-file-browser-transfer-complete = Передача завершена.
server-file-browser-modified = изменен
server-file-browser-menu-refresh = Обновить
server-file-browser-menu-upload = Загрузить
server-file-browser-menu-new = Создать
server-file-browser-menu-download = Скачать
server-file-browser-menu-upload-file = Загрузить файл
server-file-browser-menu-upload-folder = Загрузить папку
server-file-browser-menu-new-file = Новый файл
server-file-browser-menu-new-folder = Новая папка
server-file-browser-menu-copy-path = Копировать путь
server-file-browser-menu-terminal = Терминал
server-file-browser-menu-cd-to-terminal = Выполнить cd в терминале
server-file-browser-menu-other = Еще
server-file-browser-menu-copy-filename = Копировать имя файла
server-file-browser-menu-rename = Переименовать
server-file-browser-menu-delete = Удалить
server-file-browser-copied-name = Имя файла скопировано
server-file-browser-delete-title = Удалить «{ $name }»?
server-file-browser-delete-info-file = Этот файл будет удален с удаленного хоста.
server-file-browser-delete-info-directory = Эта папка и ее содержимое будут удалены с удаленного хоста.
server-file-browser-renamed = Успешно переименовано.
server-file-browser-deleted = Успешно удалено.
server-file-browser-created-file = Файл создан.
server-file-browser-created-folder = Папка создана.
server-file-browser-default-file-name = без названия
server-file-browser-default-folder-name = папка без названия
server-file-browser-rename-empty = Имя не может быть пустым.
server-file-browser-rename-invalid-name = Имя не может содержать «/».
server-file-browser-rename-unchanged = Имя не изменено.
server-file-browser-operation-failed = Не удалось выполнить операцию: { $error }
server-file-browser-rename-requires-session = Для переименования требуется подключенный удаленный сервер.
server-file-browser-create-requires-session = Для создания файлов требуется подключенный удаленный сервер.
server-file-browser-delete-requires-session = Для удаления папок требуется активная SSH-сессия.
server-file-browser-upload-progress-title = Прогресс загрузки
server-file-browser-transfer-progress-title = Прогресс передачи
server-file-browser-transfer-progress-empty = Передач пока нет.
server-file-browser-transfer-overall = Файлов: { $done } / { $total }
server-file-browser-upload-progress-empty = Загрузок пока нет.
server-file-browser-upload-status-pending = Ожидание
server-file-browser-upload-status-uploading = Загрузка { $percent }%
server-file-browser-upload-status-completed = Завершено
server-file-browser-upload-status-failed = Ошибка: { $error }
server-file-browser-download-status-pending = Ожидание
server-file-browser-download-status-downloading = Скачивание { $percent }%
server-file-browser-download-status-completed = Завершено
server-file-browser-download-status-failed = Ошибка: { $error }
server-file-browser-upload-clear-completed = Очистить завершенные
server-file-browser-upload-overall = Файлов: { $done } / { $total }
server-file-browser-upload-phase-uploading = Загрузка
server-file-browser-upload-phase-verifying = Проверка
server-file-browser-upload-phase-promoting = Применение
server-file-browser-upload-status-verifying = Проверка
server-file-browser-upload-status-promoting = Применение
server-file-browser-upload-status-skipped = Пропущено
server-file-browser-upload-all-skipped = Все файлы уже существуют; ничего не было загружено.
server-file-browser-upload-queued = Добавлено в очередь загрузки; начнется после завершения текущей задачи.
server-file-browser-upload-promote-not-replacing = Путь назначения уже существует, и перезапись не была выбрана; применить загрузку не удалось: { $path }
server-file-browser-upload-promote-not-replacing-generic = Путь назначения уже существует, и перезапись не была выбрана; применить загрузку не удалось.
server-file-browser-upload-conflict-title = Обнаружены конфликтующие пути
server-file-browser-upload-conflict-info = Следующие пути уже существуют в месте назначения. Выберите, как продолжить:
server-file-browser-upload-conflict-overwrite = Перезаписать все
server-file-browser-upload-conflict-skip = Пропустить существующие
server-file-browser-upload-conflict-kind-file = файл
server-file-browser-upload-conflict-kind-directory = папка
server-file-browser-upload-conflict-kind-symlink = символическая ссылка
server-file-browser-upload-conflict-kind-other = другое
server-file-browser-upload-conflict-more = …и еще { $count }
server-file-browser-upload-verify-missing = Проверка не пройдена: { $path } отсутствует на удаленном сервере
server-file-browser-upload-verify-size = Проверка не пройдена: размер { $path } не совпадает
workspace-left-panel-close-panel = Закрыть панель
workspace-tabs-panel-tooltip = Панель вкладок
workspace-tools-panel-tooltip = Панель инструментов
workspace-agent-management-panel-tooltip = Панель управления агентами
workspace-code-review-panel-tooltip = Панель проверки кода
workspace-notifications-tooltip = Уведомления
workspace-new-tab-tooltip = Новая вкладка
workspace-tab-configs-tooltip = Конфигурации вкладок
workspace-offline-tooltip = Некоторые функции могут быть недоступны офлайн
workspace-right-panel-open-repository = Открыть репозиторий
workspace-right-panel-open-repository-tooltip = Перейдите к репозиторию и инициализируйте его для работы с кодом
workspace-right-panel-close-panel = Закрыть панель
workspace-right-panel-code-review = Проверка кода
workspace-right-panel-minimize = Свернуть
workspace-right-panel-maximize = Развернуть
terminal-pane-new-agent-conversation-title = Новый диалог с агентом
vertical-tabs-no-tabs-open = Нет открытых вкладок
vertical-tabs-untitled-tab = Безымянная вкладка
vertical-tabs-view-options-tooltip = Параметры отображения
vertical-tabs-new-session = Новая сессия
vertical-tabs-terminal-kind-oz = Oz
vertical-tabs-pane-kind-terminal = Терминал
vertical-tabs-pane-kind-code = Код
vertical-tabs-pane-kind-code-diff = Дифф кода
vertical-tabs-pane-kind-file = Файл
vertical-tabs-pane-kind-notebook = Блокнот
vertical-tabs-pane-kind-workflow = Workflow
vertical-tabs-pane-kind-environment-variables = Переменные среды
vertical-tabs-pane-kind-environments = Окружения
vertical-tabs-pane-kind-rules = Правила
vertical-tabs-pane-kind-plan = План
vertical-tabs-pane-kind-execution-profile = Профиль выполнения
vertical-tabs-pane-kind-other = Другое
vertical-tabs-setting-view-as = Отображать как
vertical-tabs-setting-panes = Панели
vertical-tabs-setting-tabs = Вкладки
vertical-tabs-setting-tab-item = Элемент вкладки
vertical-tabs-setting-focused-session = Сессия в фокусе
vertical-tabs-setting-summary = Сводка
vertical-tabs-setting-density = Плотность
vertical-tabs-setting-pane-title-as = Заголовок панели
vertical-tabs-setting-command-conversation = Команда / Диалог
vertical-tabs-setting-working-directory = Рабочий каталог
vertical-tabs-setting-branch = Ветка
vertical-tabs-setting-additional-metadata = Дополнительные метаданные
vertical-tabs-setting-show = Показывать
vertical-tabs-setting-pr-link-requires-gh = Требуется установленный и авторизованный GitHub CLI
vertical-tabs-setting-pr-link = Ссылка на PR
vertical-tabs-setting-diff-stats = Статистика диффа
vertical-tabs-setting-show-details-on-hover = Показывать сведения при наведении
workspace-right-panel-unknown = Неизвестно
global-search-placeholder = Поиск в файлах
global-search-toggle-case-sensitivity = Переключить учет регистра
global-search-toggle-regex = Переключить регулярные выражения
global-search-label = Поиск
global-search-no-results-gitignore = Ничего не найдено. Проверьте файлы gitignore.
global-search-result-count-one = 1 результат в { $files } { $files ->
        [one] файле
        [few] файлах
        [many] файлах
       *[other] файлах
    }
global-search-result-count-many = Результатов: { $n } в { $files } { $files ->
        [one] файле
        [few] файлах
        [many] файлах
       *[other] файлах
    }
global-search-subset-warning = В результаты попала только часть совпадений. Уточните поисковый запрос, чтобы сузить результаты.
global-search-title = Глобальный поиск
global-search-description = Поиск по файлам во всех текущих каталогах.
global-search-unavailable-title = Глобальный поиск недоступен
global-search-unavailable-description = Для глобального поиска требуется доступ к локальному рабочему пространству. Откройте новую сессию или перейдите к активной сессии для просмотра.
global-search-remote-description = Для глобального поиска требуется доступ к локальному рабочему пространству, что не поддерживается в удаленных сессиях
global-search-unsupported-session-description = Глобальный поиск пока не работает в Git Bash и WSL.
global-search-failed = Не удалось выполнить глобальный поиск.

# Wasm NUX dialog (app/src/wasm_nux_dialog.rs)
wasm-nux-open-desktop-title = Открыть в Zap Desktop?
wasm-nux-open-desktop-detail = Ссылки в дальнейшем будут автоматически открываться в десктопном приложении.
wasm-nux-open-desktop-confirm = Открыть в Zap
wasm-nux-download-title = Скачать Zap Desktop?
wasm-nux-download-description = Zap — интеллектуальный терминал со встроенными AI и знаниями вашей команды разработки.
wasm-nux-learn-more = Подробнее
wasm-nux-download-confirm = Скачать
wasm-nux-object-kind-drive-objects = Объекты Zap Drive
wasm-nux-object-kind-warp-links = Ссылки Zap
wasm-nux-always-open-on-web-title = Всегда открывать { $object_kind } в веб-версии?
wasm-nux-always-open-on-web-detail = Это можно изменить в любой момент в настройках.
wasm-nux-yes = Да

# Auth override warning (app/src/auth/auth_override_warning_body.rs)
auth-override-warning-title = Обнаружен новый вход
auth-override-warning-confirm-title = Удалить личные объекты и настройки Zap Drive?
auth-override-warning-description = Похоже, вы вошли в аккаунт Zap через веб-браузер. Если вы продолжите, все личные объекты и настройки Zap Drive из этой анонимной сессии будут удалены безвозвратно.
auth-override-warning-cannot-undo = Это действие нельзя отменить.
auth-override-warning-export = Экспортируйте данные
auth-override-warning-export-description = , чтобы импортировать позже.
auth-override-warning-cancel = Отмена
auth-override-warning-continue = Продолжить
auth-override-warning-accessibility-help = Zap обнаружил новый вход через веб-браузер. Нажмите Escape, чтобы отменить и продолжить работу в Zap без входа.

# Auth SSO link/login failures/paste token/logout/offline/privacy
auth-needs-sso-link-button = Связать SSO
auth-needs-sso-link-title = Ваша организация включила SSO для вашего аккаунта
auth-needs-sso-link-detail = Нажмите кнопку ниже, чтобы связать аккаунт Zap с вашим SSO-провайдером.
auth-login-failure-troubleshooting-prefix =  Не в первый раз? См. нашу
auth-login-failure-troubleshooting-link =  документацию по устранению неполадок
auth-login-failure-troubleshooting-suffix = .
auth-login-failure-invalid-token = В модальное окно введен недействительный токен авторизации.
auth-login-failure-copy-token-manually = Не удалось войти. Попробуйте вручную скопировать токен авторизации со страницы аутентификации и вставить его в модальное окно.
auth-login-failure-login-request = Не удалось выполнить запрос на вход.
auth-login-failure-signup-request = Не удалось выполнить запрос на регистрацию.
auth-login-failure-wrong-redirect-url = Вставленный URL переадресации исходит не от этого приложения. Нажмите кнопку ниже, чтобы попробовать снова.
auth-paste-token-placeholder = Введите токен авторизации
auth-paste-token-title = Вставьте токен авторизации ниже
auth-paste-token-detail = Вставьте токен авторизации из браузера, чтобы завершить вход.
auth-paste-token-cancel = Отмена
auth-paste-token-continue = Продолжить
auth-offline-first-use-description = Сейчас вы офлайн. Для первого использования Zap требуется подключение к интернету.
auth-offline-first-use-learn-more = Подробнее
auth-offline-overlay-title = Использование Zap офлайн
auth-offline-overlay-paragraph-1 = Zap можно использовать офлайн для локальной работы в терминале и с агентами.
auth-offline-overlay-paragraph-2 = Некоторые этапы настройки все же могут требовать подключения к интернету, если они зависят от внешних провайдеров.
auth-offline-overlay-paragraph-3 = При работе без входа локальные процессы остаются на этом компьютере.
auth-offline-overlay-dismiss = Закрыть
auth-privacy-settings-title = Настройки конфиденциальности
auth-privacy-settings-done = Готово
auth-privacy-settings-help-improve = Помогите улучшить Zap
auth-privacy-settings-help-improve-description = Обобщенные данные об использовании функций помогают продуктовой команде Zap расставлять приоритеты в развитии.
auth-privacy-settings-learn-more = Подробнее
auth-privacy-settings-send-crash-reports = Отправлять отчеты о сбоях
auth-privacy-settings-crash-reports-description = Отчеты о сбоях помогают команде разработки Zap следить за стабильностью и улучшать производительность.
auth-logout-confirm = Да, выйти
auth-logout-show-running-processes = Показать запущенные процессы
auth-logout-cancel = Отмена
auth-logout-title = Выйти из аккаунта?
auth-logout-running-processes-warning = У вас { $count } { $count ->
        [one] запущенный процесс
        [few] запущенных процесса
        [many] запущенных процессов
       *[other] запущенного процесса
    }.
auth-logout-shared-sessions-warning = У вас { $count } { $count ->
        [one] удаленная сессия
        [few] удаленные сессии
        [many] удаленных сессий
       *[other] удаленной сессии
    }.
auth-logout-unsynced-drive-objects-warning = У вас { $count } { $count ->
        [one] несинхронизированный объект
        [few] несинхронизированных объекта
        [many] несинхронизированных объектов
       *[other] несинхронизированного объекта
    } Zap Drive. При выходе из аккаунта вы потеряете { $count ->
        [one] объект
        [few] объекта
        [many] объектов
       *[other] объекта
    }.
auth-logout-unsaved-files-warning = У вас { $count } { $count ->
        [one] несохраненный файл
        [few] несохраненных файла
        [many] несохраненных файлов
       *[other] несохраненного файла
    }. При выходе из аккаунта вы потеряете { $count ->
        [one] файл
        [few] файла
        [many] файлов
       *[other] файла
    }.

# CLI agent plugin instructions
cli-agent-plugin-run-on-remote = Обязательно выполняйте эти команды на удаленной машине.
cli-agent-plugin-codex-install-title = Включить уведомления Zap для Codex
cli-agent-plugin-codex-install-subtitle = Обновите Codex до последней версии, затем включите уведомления в фокусе, чтобы Zap мог показывать их во время вашей работы.
cli-agent-plugin-codex-update-step = Обновите Codex до последней версии.
cli-agent-plugin-codex-notification-step = Установите условие уведомлений "always" в конфигурации Codex. Откройте или создайте ~/.codex/config.toml и добавьте:
cli-agent-plugin-codex-restart-note = Перезапустите Codex, чтобы применить изменения.
cli-agent-plugin-deepseek-install-title = Включить уведомления Zap для DeepSeek
cli-agent-plugin-deepseek-install-subtitle = Добавьте следующее в файл конфигурации DeepSeek (~/.deepseek/config.toml), чтобы включить уведомления о завершении хода.
cli-agent-plugin-deepseek-notification-step = Установите условие уведомлений "always" в ~/.deepseek/config.toml:
cli-agent-plugin-deepseek-restart-note = Перезапустите DeepSeek, чтобы применить изменения.
cli-agent-plugin-claude-install-title = Установить плагин Zap для Claude Code
cli-agent-plugin-claude-install-subtitle = Убедитесь, что jq установлен на вашей машине. Затем выполните эти команды.
cli-agent-plugin-claude-add-marketplace-step = Добавьте репозиторий маркетплейса плагинов Warp
cli-agent-plugin-install-warp-plugin-step = Установите плагин Warp
cli-agent-plugin-claude-restart-note = Перезапустите Claude Code, чтобы активировать плагин.
cli-agent-plugin-claude-known-issues-note = В системе плагинов Claude Code есть известные проблемы. Если плагин не найден после шага 1, попробуйте вручную добавить запись "extraKnownMarketplaces" в ~/.claude/settings.json.
cli-agent-plugin-claude-update-title = Обновить плагин Zap для Claude Code
cli-agent-plugin-run-following-commands = Выполните следующие команды.
cli-agent-plugin-remove-existing-marketplace-step = Удалите существующий маркетплейс (если он есть)
cli-agent-plugin-readd-marketplace-step = Добавьте маркетплейс заново
cli-agent-plugin-install-latest-version-step = Установите последнюю версию плагина
cli-agent-plugin-claude-restart-update-note = Перезапустите Claude Code, чтобы применить обновление.
cli-agent-plugin-gemini-install-title = Установить плагин Zap для Gemini CLI
cli-agent-plugin-gemini-run-command-restart = Выполните следующую команду, затем перезапустите Gemini CLI.
cli-agent-plugin-install-warp-extension-step = Установите расширение Warp
cli-agent-plugin-gemini-restart-note = Перезапустите Gemini CLI, чтобы активировать плагин.
cli-agent-plugin-gemini-update-title = Обновить плагин Zap для Gemini CLI
cli-agent-plugin-update-warp-extension-step = Обновите расширение Warp
cli-agent-plugin-gemini-restart-update-note = Перезапустите Gemini CLI, чтобы применить обновление.
cli-agent-plugin-opencode-install-title = Установить плагин Zap для OpenCode
cli-agent-plugin-opencode-install-subtitle = Добавьте плагин Warp в конфигурацию OpenCode, затем перезапустите OpenCode.
cli-agent-plugin-opencode-open-config-step = Откройте или создайте файл opencode.json. Он может находиться в корне проекта или по пути глобальной конфигурации:
cli-agent-plugin-opencode-add-plugin-step = Добавьте "@warp-dot-dev/opencode-warp" в массив "plugin" в объекте верхнего уровня JSON:
cli-agent-plugin-opencode-restart-note = Перезапустите OpenCode, чтобы активировать плагин.
cli-agent-plugin-opencode-update-title = Обновить плагин Zap для OpenCode
cli-agent-plugin-opencode-update-subtitle = Закрепите плагин за последней версией в opencode.json. OpenCode кэширует плагины по спецификации версии, поэтому изменение закрепленной версии заставит его заново загрузить плагин при перезапуске.
cli-agent-plugin-opencode-replace-plugin-step = Замените существующую запись "@warp-dot-dev/opencode-warp" в массиве "plugin" на явную версию:
cli-agent-plugin-opencode-restart-update-note = Перезапустите OpenCode, чтобы загрузить обновленный плагин.

# Remaining visible UI strings
ai-ask-user-questions-unavailable = Вопросы недоступны
ai-ask-user-questions-skipped-auto-approve = Вопросы пропущены из-за автоподтверждения
terminal-bootstrapping-checking = Проверка…
terminal-bootstrapping-installing-progress = Установка… ({ $p }%)
terminal-bootstrapping-installing = Установка…
terminal-bootstrapping-updating = Обновление…
terminal-bootstrapping-initializing = Инициализация…
terminal-bootstrapping-installing-warp-ssh-extension-progress = Установка расширения Zap SSH… ({ $p }%)
terminal-bootstrapping-installing-warp-ssh-extension = Установка расширения Zap SSH…
terminal-bootstrapping-updating-warp-ssh-extension = Обновление расширения Zap SSH…
terminal-bootstrapping-starting-shell-name = Запуск { $shell }…
agent-tip-prefix = Совет:
agent-tip-slash-menu = `/` — открыть меню слэш-команд и получить доступ к быстрым действиям агента.
agent-tip-toggle-input-mode = <keybinding> — переключить определение естественного языка и менять режим ввода между агентом и терминалом.
agent-tip-plan = `/plan` <prompt> — создать план для агента перед выполнением.
agent-tip-command-palette = <keybinding> — открыть палитру команд и получить доступ к действиям и сочетаниям клавиш Zap.
agent-tip-warp-drive = Храните повторно используемые Workflow, блокноты и промпты в вашем
agent-tip-redirect-running-agent = Введите новый промпт, чтобы перенаправить агента во время его работы.
agent-tip-add-context = `@` — добавить в промпт контекст из файлов, блоков или объектов Zap Drive.
agent-tip-attach-prior-output = <keybinding> — прикрепить вывод предыдущей команды как контекст для агента.
agent-tip-init-index = `/init` — проиндексировать репозиторий, чтобы агент понимал вашу кодовую базу.
agent-tip-agent-profiles = Добавляйте профили агентов, чтобы настраивать разрешения и модели для каждой сессии.
agent-tip-fork-block = Щелкните блок правой кнопкой, чтобы ответвить диалог с этого места.
agent-tip-copy-output = Щелкните блок правой кнопкой, чтобы скопировать вывод диалога.
agent-tip-drag-image = Перетащите изображение в панель, чтобы прикрепить его как контекст для агента.
agent-tip-interactive-tools = Попросите агента управлять интерактивными инструментами вроде node, python, postgres, gdb или vim.
agent-tip-code-review-panel = <keybinding> — открыть панель проверки кода и просмотреть изменения агента.
agent-tip-add-mcp = `/add-mcp` — добавить MCP-сервер в рабочее пространство.
agent-tip-open-mcp-servers = `/open-mcp-servers` — просматривать локальные MCP-серверы и управлять ими.
agent-tip-add-prompt = `/add-prompt` — создать повторно используемый промпт для типовых Workflow.
agent-tip-add-rule = `/add-rule` — создать глобальное правило для агента.
agent-tip-fork = `/fork` — создать копию текущего диалога, при необходимости с новым промптом.
agent-tip-open-code-review = `/open-code-review` — открыть панель проверки кода и изучить диффы, созданные агентом.
agent-tip-new-conversation = `/new` — начать новый диалог с агентом с чистым контекстом.
agent-tip-compact = `/compact` — создать сводку текущего диалога и освободить место в контекстном окне.
agent-tip-usage = `/usage` — показать текущее использование AI-кредитов.
agent-tip-oz-headless = Используйте команду `oz`, чтобы запустить агента Oz в headless-режиме — это полезно для удаленных машин.
agent-tip-selected-text-context = Щелкните выделенный текст правой кнопкой, чтобы прикрепить его как контекст для агента.
agent-tip-project-rules = Используйте `AGENTS.md` или `CLAUDE.md`, чтобы применять правила в рамках проекта.
agent-tip-url-context = Вставьте URL, чтобы прикрепить эту веб-страницу как контекст для агента.
agent-tip-warpify-ssh = Warpify удаленной SSH-сессии, чтобы включить Oz в этом окружении.
agent-tip-switch-profiles = Переключайте профили агента, чтобы быстро менять модели и разрешения агента.
agent-tip-init-rules = `/init` — создать файл `WARP.md` и определить правила проекта для агента.
agent-tip-auto-approve = <keybinding> — автоматически одобрять команды и диффы агента до конца сессии.
agent-tip-desktop-notifications = Включите уведомления на рабочем столе, чтобы получать оповещения, когда агент требует вашего внимания.
agent-tip-cancel-task = <keybinding> — отменить текущую задачу агента.
agent-tip-action-open-palette = Открыть палитру
agent-tip-action-warp-drive = Zap Drive.
agent-tip-action-show-diff-view = Показать режим диффа
agent-tip-voice-input = Удерживайте <keybinding>, чтобы произнести промпт прямо агенту.
hoa-welcome-banner-title = Представляем универсальную поддержку агентов: усильте любого coding-агента с Zap
hoa-feature-vertical-tabs-title = Вертикальные вкладки
hoa-feature-vertical-tabs-description = Богатые заголовки вкладок и метаданные: ветка git, worktree и PR. Полностью настраивается.
hoa-feature-tab-configs-title = Конфигурации вкладок
hoa-feature-tab-configs-description = Схема на уровне вкладки, чтобы одним кликом задать каталог, команды запуска, тему и worktree
hoa-feature-agent-inbox-title = Входящие агента
hoa-feature-agent-inbox-description = Уведомления, когда любому агенту нужно ваше внимание, также доступны в едином входящем
hoa-feature-native-code-review-title = Нативный Code Review
hoa-feature-native-code-review-description = Отправляйте встроенные комментарии из Code Review Zap прямо в Claude Code, Codex или OpenCode
resource-center-whats-new-section = Что нового?
resource-center-getting-started-section = Начало работы
resource-center-maximize-warp-section = Максимум от Zap
resource-center-advanced-setup-section = Расширенная настройка
resource-center-create-first-block-title = Создайте свой первый блок
resource-center-create-first-block-description = Выполните команду, чтобы увидеть команду и вывод, сгруппированные вместе.
resource-center-navigate-blocks-title = Навигация по блокам
resource-center-navigate-blocks-description = Щелкните, чтобы выбрать блок, и перемещайтесь с помощью стрелок.
resource-center-block-action-title = Действие над блоком
resource-center-block-action-description = Щелкните по блоку правой кнопкой, чтобы копировать, вставить, поделиться и другое.
resource-center-command-palette-title = Откройте палитру команд
resource-center-command-palette-description = Доступ ко всему в Zap с клавиатуры.
resource-center-set-theme-title = Задайте свою тему
resource-center-set-theme-description = Сделайте Zap своим, выбрав тему.
resource-center-custom-prompt-title = Используйте свое приглашение shell
resource-center-custom-prompt-description = Настройте Zap, чтобы учитывать ваше значение PS1
resource-center-view-documentation = Посмотреть документацию
resource-center-integrate-ide-title = Интегрируйте Zap с вашей IDE
resource-center-integrate-ide-description = Настройте запуск Zap из наиболее часто используемых инструментов разработки
resource-center-how-warp-uses-warp-title = Как Zap использует Zap
resource-center-how-warp-uses-warp-description = Узнайте, как команда разработки Zap использует любимые функции
resource-center-read-article = Читать статью
resource-center-command-search-title = Поиск команд
resource-center-command-search-description = Находите и запускайте ранее выполненные команды, workflow и другое.
resource-center-ai-command-search-title = AI-поиск команд
resource-center-ai-command-search-description = Генерируйте команды shell на естественном языке.
resource-center-split-panes-title = Разделение панелей
resource-center-split-panes-description = Разделяйте вкладки на несколько панелей, чтобы создать идеальную раскладку.
resource-center-launch-configuration-title = Конфигурация запуска
resource-center-launch-configuration-description = Сохраните текущую конфигурацию окон, вкладок и панелей.
notebook-link-new-session = Новая сессия
notebook-link-new-session-tooltip = Открыть новую сессию терминала в этом каталоге
notebook-link-open-terminal-session = Открыть в сессии терминала
notebook-link-open-in-editor = Открыть в редакторе
notebook-link-edit-markdown-file = Редактировать файл Markdown
auth-token-placeholder = Токен авторизации
sharing-inherited-from-prefix = Унаследовано от{" "}
sharing-inherited-permission-label = Унаследованное разрешение
sharing-inherited-permissions-edit-parent-tooltip = Редактировать унаследованные разрешения в родительской папке
sharing-inherited-permissions-cannot-edit-tooltip = Нельзя редактировать унаследованные разрешения
command-palette-navigation-running = Выполняется...
command-palette-navigation-completed-over-hour = Завершено больше часа назад
command-palette-navigation-completed-minute-ago = Завершено { $mins } минуту назад
command-palette-navigation-completed-minutes-ago = Завершено { $mins } минут назад
command-palette-navigation-no-timestamp = Метка времени не найдена
command-palette-navigation-completed = Завершено
command-palette-navigation-empty-session = Пустая сессия
terminal-history-tab-commands = Команды
terminal-history-tab-prompts = Промпты
common-current = Текущая
auth-browser-token-placeholder = Токен авторизации в браузере
requested-script-expand-to-show = Развернуть, чтобы показать скрипт
common-hide = Скрыть
terminal-message-new-conversation = {" "}новый разговор
agent-message-bar-again-send-to-agent = еще раз, чтобы отправить агенту

# =============================================================================
# SECTION: remaining-ui-surfaces (Owner: codex-i18n-remaining-ui-surfaces)
# Files: onboarding slides, auth modal, voice, launch configs, notebook file state,
#        resource center, theme picker, terminal banners, AI footer/tool output
# =============================================================================

onboarding-intention-title = Добро пожаловать в Zap
onboarding-intention-subtitle = Как вы хотите работать?
onboarding-intention-agent-title = Работайте быстрее с AI-агентами
onboarding-intention-agent-description = Опыт с приоритетом агентов и поддержкой терминала лучшего класса. Получите терминальные и агентные AI-функции для разработки, такие как:
onboarding-intention-terminal-title = Просто пользоваться терминалом
onboarding-intention-terminal-badge = Без AI-функций
onboarding-intention-terminal-description = Современный терминал, оптимизированный для скорости, контекста и контроля, без AI.
onboarding-ai-feature-warp-agents = Агенты Zap
onboarding-ai-feature-oz-cloud-agents-platform = Платформа локальных агентов Oz
onboarding-ai-feature-next-command-predictions = Предсказание следующей команды
onboarding-ai-feature-prompt-suggestions = Подсказки промптов
onboarding-ai-feature-remote-control-agents = Удаленное управление с Claude Code, Codex и другими агентами
onboarding-ai-feature-agents-over-ssh = Агенты по SSH
onboarding-agent-title = Настройте своего агента Zap
onboarding-agent-subtitle = Выберите параметры агента в приложении по умолчанию.
onboarding-agent-default-model = Модель по умолчанию
onboarding-agent-autonomy = Автономность
onboarding-agent-set-by-team-workspace = Управляется локальной политикой рабочего пространства
onboarding-agent-team-workspace-autonomy-description = Настройки автономности задаются локальной политикой рабочего пространства.
onboarding-agent-autonomy-full-title = Полная
onboarding-agent-autonomy-full-subtitle = Выполняет команды, пишет код и читает файлы, не спрашивая.
onboarding-agent-autonomy-partial-title = Частичная
onboarding-agent-autonomy-partial-subtitle = Может планировать, читать файлы и выполнять низкорисковые команды. Спрашивает перед любыми изменениями или выполнением чувствительных команд.
onboarding-agent-autonomy-none-title = Нет
onboarding-agent-autonomy-none-subtitle = Не выполняет никаких действий без вашего одобрения.
onboarding-agent-disable-warp-agent = Отключить агента Zap
onboarding-project-title = Откройте проект
onboarding-project-subtitle = Настройте проект, чтобы оптимизировать его для разработки в Zap.
onboarding-project-open-local-folder = Открыть локальную папку
onboarding-project-initialize-automatically = Инициализировать проект автоматически
onboarding-project-initialize-description = Готовит окружение проекта, строит индекс вашего кода и генерирует правила проекта — это дает агенту более глубокое понимание и лучшую производительность.
onboarding-intro-already-have-account = Уже есть аккаунт?{" "}
onboarding-intro-subtitle = Современный терминал со встроенными агентами лучшего в своем классе уровня.
onboarding-get-started = Начать
onboarding-theme-title = Выберите тему
onboarding-theme-subtitle = Щелкните или используйте стрелки для выбора, Enter для подтверждения.
onboarding-theme-sync-with-os = Синхронизировать светлую/темную тему с ОС
onboarding-third-party-title = Настройте сторонних агентов
onboarding-third-party-subtitle = Выберите параметры по умолчанию для агентов вроде Claude Code, Codex и Gemini.
onboarding-third-party-cli-toolbar = Панель инструментов CLI-агента
onboarding-third-party-notifications = Уведомления
onboarding-customize-title = Настройте свой Zap
onboarding-customize-subtitle = Адаптируйте функции и интерфейс под свой стиль работы.
onboarding-customize-tab-styling = Оформление вкладок
onboarding-customize-vertical = Вертикально
onboarding-customize-horizontal = Горизонтально
onboarding-customize-conversation-history = История разговоров
onboarding-customize-file-explorer = Проводник файлов
onboarding-customize-global-file-search = Глобальный поиск по файлам
onboarding-customize-warp-drive = Zap Drive
onboarding-customize-tools-panel = Панель инструментов
onboarding-customize-code-review = Code Review

auth-opt-out-line-1 = Zap хранит выбранные настройки Onboarding локально.
auth-opt-out-line-2-prefix = Вы можете изменить свои{" "}
auth-privacy-settings-prefix = Вы можете изменить свои{" "}
auth-privacy-settings-ai-prefix = Вы можете изменить свои локальные настройки AI в{" "}
auth-privacy-settings = Настройки приватности
auth-local-privacy-note = Zap хранит выбранные настройки Onboarding локально на этом устройстве.
auth-terms-prefix = Продолжая, вы сохраняете эту настройку на своем устройстве.{" "}
auth-terms-of-service = Локальная настройка
auth-log-in = Войти
auth-paste-token-from-browser = Нажмите здесь, чтобы вставить токен из браузера
auth-login-slide-title-warp-drive = Начните с Zap Drive
auth-login-slide-title-ai = Начните с AI
auth-login-slide-subtitle-warp-drive = Подключите аккаунт, чтобы сохранять и делиться блокнотами, workflow и другим между устройствами.
auth-login-slide-subtitle-ai = Подключите аккаунт, чтобы включить планирование, написание кода и автоматизацию на базе AI.
auth-disable-warp-drive = Отключить Zap Drive
auth-disable-ai-features = Отключить AI-функции
auth-enable-warp-drive = Включить Zap Drive
auth-enable-ai-features = Включить AI-функции
auth-browser-sign-in-one-line-title = Войдите через браузер, чтобы продолжить
auth-open-page-manually-line-prefix = {" "}и откройте
auth-open-page-manually-line-suffix = страницу вручную.
auth-disable-warp-drive-confirm-title = Вы уверены, что хотите отключить Zap Drive?
auth-disable-ai-features-confirm-title = Вы уверены, что хотите отключить AI-функции?
auth-disable-warp-drive-confirm-body = Zap Drive позволяет сохранять workflow и знания между устройствами и делиться ими с командой. Если вы продолжите, у вас не будет доступа к следующим функциям:
auth-disable-ai-features-confirm-body = Zap лучше с AI. Если вы продолжите, у вас не будет доступа ни к одной из следующих функций:
auth-feature-session-sharing = Обмен сессиями
auth-sign-up = Продолжить локально
auth-sign-in = Войти
auth-already-have-account = Уже есть аккаунт?{" "}
auth-dont-want-sign-in-now = Не хотите входить прямо сейчас?{" "}
auth-skip-for-now = Пропустить пока
auth-skip-login-confirm-title = Вы уверены, что хотите пропустить вход?
auth-skip-login-confirm-line-1 = Вы можете зарегистрироваться позже, но некоторые функции, например AI,
auth-skip-login-confirm-line-2-prefix = доступны только вошедшим пользователям.{" "}
auth-yes-skip-login = Да, пропустить вход
auth-require-login-ai-collaboration = Локальные AI-функции не требуют аккаунт Zap.
auth-require-login-drive-limit = Объекты Zap Drive хранятся локально в Zap.
auth-require-login-share = Обмен недоступен в локальных сборках Zap.
auth-welcome-title = Добро пожаловать в Zap!
auth-sign-up-for-warp = Продолжить в Zap
auth-browser-sign-in-title = Войдите через браузер,\nчтобы продолжить
auth-open-page-manually-suffix = и откройте страницу вручную.

voice-try-input = Попробуйте голосовой ввод
voice-input-enabled-toast = Голосовой ввод включен. Вы также можете нажать и удерживать клавишу `{ $key }`, чтобы активировать голосовой ввод (настройка в Настройки > AI > Голос)
voice-input-microphone-access-error = Не удалось запустить голосовой ввод (возможно, нужно разрешить доступ к микрофону)
voice-transcription-disabled-microphone = Транскрипция голоса отключена, так как доступ к микрофону не был предоставлен.
voice-transcription = Транскрипция голоса
voice-transcription-hold-key = Транскрипция голоса (удерживайте клавишу `{ $key }`)

get-started-welcome-title = Добро пожаловать в Zap
get-started-subtitle = Среда агентной разработки
theme-creator-theme-name = Название темы
theme-creator-background-color = Цвет фона
theme-creator-image-subheader = Автоматически создать тему на основе цветов, извлеченных из изображения (.png, .jpg).
theme-creator-select-image = Выберите изображение
theme-creator-selecting-image = Выбираем изображение...
theme-creator-select-new-image = Выбрать другое изображение
theme-creator-create-theme = Создать тему
theme-creator-process-image-failed = Не удалось обработать выбранное изображение. Попробуйте другое изображение.
theme-chooser-current-description = Измените текущую тему.
theme-chooser-light-description = Выберите тему на случай, когда система находится в светлом режиме.
theme-chooser-dark-description = Выберите тему на случай, когда система находится в темном режиме.
theme-chooser-no-matching-themes = Нет подходящих тем!
resource-center-keyboard-shortcuts = Сочетания клавиш
resource-center-keybindings-essentials = Основное
resource-center-keybindings-blocks = Блоки
resource-center-keybindings-input-editor = Редактор ввода
resource-center-keybindings-terminal = Терминал
resource-center-keybindings-fundamentals = Базовые основы

launch-config-save-success-prefix = Успешно сохранено в{" "}
launch-config-save-failure-already-exists = Не удалось сохранить. Конфигурация запуска с таким именем уже существует.
launch-config-save-failure-other = При сохранении возникла проблема.
launch-config-save-configuration = Сохранить конфигурацию
launch-config-open-yaml-file = Открыть YAML-файл
launch-config-save-current-configuration = Сохранить текущую конфигурацию
launch-config-link-to-documentation = Ссылка на документацию
launch-config-save-modal-a11y-title = Модальное окно сохранения конфигурации
launch-config-save-modal-a11y-description = Введите имя файла, в который вы хотите сохранить текущую конфигурацию окон, вкладок и панелей. Нажмите Enter, чтобы сохранить конфигурацию запуска, Esc — чтобы закрыть окно сохранения.
launch-config-save-description-no-keybinding = Это сохранит текущую конфигурацию окон, вкладок и панелей в файл, чтобы вы могли легко открыть ее снова.
launch-config-save-description-with-keybinding = Это сохранит текущую конфигурацию окон, вкладок и панелей в файл, чтобы вы могли легко открыть ее снова с помощью { $keybinding }.
launch-config-yaml-saved-to-prefix = \nYAML-файл сохранен в{" "}
notebook-file-could-not-read = Не удалось прочитать { $name }
notebook-file-loading = Загрузка { $name }...
notebook-file-missing-source = Исходный файл отсутствует

terminal-shared-session-reconnecting = Нет соединения, пытаемся переподключиться...
terminal-banner-p10k-supported = Powerlevel10k теперь поддерживает Zap!{"  "}
terminal-banner-p10k-older-version-prefix = Похоже, вы используете более старую (неподдерживаемую) версию, пожалуйста, следуйте{" "}
terminal-banner-these-instructions = этим инструкциям,
terminal-banner-update-latest-suffix = {" "}чтобы обновиться до последней версии.
terminal-banner-pure-unsupported = Pure пока не поддерживается в Zap. Возможно, вы рассмотрете одно из поддерживаемых приглашений shell как альтернативу.{"  "}
terminal-loading-session = Загрузка сессии...

ai-footer-hide-rich-input = Скрыть расширенный ввод
ai-footer-choose-environment = Выберите окружение
ai-footer-agent-environment = Окружение агента
ai-footer-enable-terminal-command-autodetection = Включить автоопределение команд терминала
ai-footer-disable-terminal-command-autodetection = Отключить автоопределение команд терминала
ai-footer-turn-off-auto-approve-agent-actions = Отключить автоодобрение всех действий агента
ai-footer-auto-approve-agent-actions-for-task = Автоодобрять все действия агента для этой задачи
ai-footer-start-remote-control = Запустить удаленное управление
ai-footer-login-required-remote-control = Войдите, чтобы использовать /remote-control
ai-footer-see-logs-for-details = Подробности в журналах
ai-footer-plugin-installed-restart-session = Плагин Warp установлен. Перезапустите сессию, чтобы активировать его.
ai-footer-installing-warp-plugin = Устанавливаем плагин Warp...
ai-footer-failed-install-warp-plugin = Не удалось установить плагин Warp
ai-footer-plugin-updated-restart-session = Плагин Warp обновлен. Перезапустите сессию, чтобы активировать его.
ai-footer-updating-warp-plugin = Обновляем плагин Warp...
ai-footer-failed-update-warp-plugin = Не удалось обновить плагин Warp
voice-input-limit-reached = Достигнут лимит голосового ввода
voice-input-transcription-failed = Не удалось выполнить транскрипцию голосового ввода
ai-toolbar-context-chip = Чип контекста
ai-toolbar-model-selector = Селектор модели
ai-toolbar-autodetection = Автоопределение
ai-toolbar-voice-input = Голосовой ввод
ai-toolbar-attach-file = Прикрепить файл
ai-toolbar-context-usage = Использование контекста
ai-toolbar-file-explorer = Проводник файлов
ai-toolbar-rich-input = Расширенный ввод
ai-toolbar-fast-forward = Быстрая перемотка
ai-tool-output-grep-for = Grep по{" "}
ai-tool-output-grepping-for = Выполняется grep по{" "}
ai-tool-output-in-path-cancelled = {" "}в { $path } отменен
ai-tool-output-in-path = {" "}в { $path }
ai-tool-output-grep-patterns-cancelled = Отменен grep по следующим шаблонам в { $path }
ai-tool-output-grep-patterns-queued = Grep по следующим шаблонам в { $path }
ai-tool-output-grep-patterns-running = Выполняется grep по следующим шаблонам в { $path }
ai-tool-output-search-files-match = Поиск файлов, соответствующих{" "}
ai-tool-output-finding-files-match = Выполняется поиск файлов, соответствующих{" "}
ai-tool-output-file-patterns-cancelled = Отменен поиск файлов, соответствующих следующим шаблонам в { $path }
ai-tool-output-file-patterns-queued = Найти файлы, соответствующие следующим шаблонам в { $path }
ai-tool-output-file-patterns-running = Выполняется поиск файлов, соответствующих следующим шаблонам в { $path }
ai-tool-output-listing-messages = Вывод списка сообщений
ai-tool-output-grepping-patterns = Выполняется grep по шаблонам
ai-tool-output-grepping-patterns-with-query = Выполняется grep по шаблонам: { $query }
ai-tool-output-reading-messages = Чтение сообщений: { $count }

code-review-discard-uncommitted-changes-title = Отменить незакоммиченные изменения?
code-review-discard-file-uncommitted-changes-title = Отменить все незакоммиченные изменения в файле?
code-review-discard-all-changes-title = Отменить все изменения?
code-review-discard-file-changes-title = Отменить все изменения в файле?
code-review-discard-uncommitted-changes-description = Вы собираетесь отменить все локальные изменения, которые еще не были закоммичены.
code-review-discard-file-uncommitted-changes-description = Файл будет восстановлен до последней закоммиченной версии, а локальные правки будут отменены.
code-review-discard-all-changes-description = Вы собираетесь отменить все закоммиченные и незакоммиченные изменения.
code-review-discard-file-main-branch-description = Файл будет восстановлен до версии из ветки main, а все закоммиченные и незакоммиченные правки будут отменены.
code-review-discard-file-branch-description = Файл будет сброшен до версии из ветки { $branch }, а все закоммиченные и незакоммиченные правки будут отменены.
code-review-stash-changes = Спрятать изменения
code-review-no-changes-to-commit = Нет изменений для коммита
code-review-no-git-actions-available = Нет доступных действий git
command-search-out-of-credits-contact-admin = Похоже, у вас закончились кредиты. Обратитесь к администратору команды, чтобы улучшить тариф и получить больше кредитов.
command-search-out-of-credits-prefix = Похоже, у вас закончились кредиты.{" "}
command-search-for-more-credits-suffix = {" "}чтобы получить больше кредитов.
search-not-visible-to-other-users = Не виден другим пользователям
sharing-invite = Пригласить
sharing-who-has-access = У кого есть доступ
terminal-shared-session-cancel-request = Отменить запрос
terminal-shared-session-continue-sharing = Продолжить общий доступ
settings-import-reset-to-warp-defaults = Сбросить к настройкам Zap по умолчанию
settings-import-type-theme = Тема
settings-import-type-theme-with-comma = Тема,
settings-import-type-option-as-meta = Option как Meta
settings-import-type-mouse-scroll-reporting = Отчеты мыши/прокрутки
settings-import-type-font = Шрифт
settings-import-type-default-shell = Shell по умолчанию
settings-import-type-working-directory = Рабочий каталог
settings-import-type-global-hotkey = Глобальная горячая клавиша
settings-import-type-window-dimensions = Размеры окна
settings-import-type-copy-on-select = Копирование при выделении
settings-import-type-window-opacity = Прозрачность окна
settings-import-type-cursor-blinking = Мигание курсора
settings-import-one-other-setting = еще 1 настройка
settings-import-other-settings = еще { $count } настроек
workflow-argument-editor-helper = Заполните аргументы в этом workflow и скопируйте его для запуска в вашей сессии терминала
workflow-add-environment-variables = Добавить переменные окружения
workflow-environment-variables = Переменные окружения
workflow-new-environment-variables = Новые переменные окружения
ai-history-completed-successfully = Успешно завершено
ai-history-pending = В ожидании
ai-history-cancelled-by-user = Отменено пользователем
ai-block-always-allow = Всегда разрешать
ai-cancel-summarization = Отменить суммаризацию
ai-continue-summarization = Продолжить суммаризацию
ai-dont-show-suggested-code-banners-again = Больше не показывать мне баннеры с предлагаемым кодом
ai-inline-code-diff-no-file-name = Без имени файла
ai-tool-call-cancelled = Вызов инструмента был отменен
ai-agent-view-open-in-different-pane = Открыть в другой панели
passive-suggestion-feature-or-bug-label = Разработать функцию или исправить ошибку в {1}
passive-suggestion-help-feature-or-bug-label = Помоги мне разработать функцию или исправить ошибку в {1}
passive-suggestion-implement-feature-or-bug-query = Реализуй функцию или исправь ошибку в {1}. Спроси у меня все необходимые подробности.
passive-suggestion-create-pull-request-query = Помоги мне создать pull request.
passive-suggestion-start-new-project-label = Помоги мне начать новый проект
passive-suggestion-start-new-project-query = Помоги мне начать новый проект. Спроси у меня все необходимые подробности.
passive-suggestion-node-project-label = Помоги мне начать проект на Node.js
passive-suggestion-node-project-query = Помоги мне начать проект на Node.js. Спроси у меня все необходимые подробности.
passive-suggestion-react-app-label = Помоги мне создать новое React-приложение
passive-suggestion-react-app-query = Помоги мне создать новое React-приложение под названием {1}. Спроси у меня все необходимые подробности.
passive-suggestion-next-app-label = Помоги мне создать новое Next.js-приложение
passive-suggestion-next-app-query = Помоги мне создать новое Next.js-приложение под названием {1}. Спроси у меня все необходимые подробности.
passive-suggestion-rust-project-label = Помоги мне начать проект на Rust для {1}
passive-suggestion-rust-project-query = Помоги мне начать проект на Rust для {1}. Спроси у меня все необходимые подробности.
passive-suggestion-poetry-project-label = Помоги мне начать проект на Poetry для {1}
passive-suggestion-poetry-project-query = Помоги мне начать проект на Poetry для {1}. Спроси у меня все необходимые подробности.
passive-suggestion-django-project-label = Помоги мне начать проект на Django для {1}
passive-suggestion-django-project-query = Помоги мне начать проект на Django для {1}. Спроси у меня все необходимые подробности.
passive-suggestion-rails-app-label = Помоги мне создать приложение на Rails для {1}
passive-suggestion-rails-app-query = Помоги мне создать приложение на Rails для {1}. Спроси у меня все необходимые подробности.
passive-suggestion-gradle-maven-project-label = Помоги мне начать проект на Gradle/Maven
passive-suggestion-gradle-maven-project-query = Помоги мне начать проект на Gradle/Maven. Спроси у меня все необходимые подробности.
passive-suggestion-go-project-label = Помоги мне начать проект на Go для {1}
passive-suggestion-go-project-query = Помоги мне начать проект на Go для {1}. Спроси у меня все необходимые подробности.
passive-suggestion-swift-project-label = Помоги мне начать проект на Swift
passive-suggestion-swift-project-query = Помоги мне начать проект на Swift. Спроси у меня все необходимые подробности.
passive-suggestion-terraform-config-label = Помоги мне создать конфигурацию Terraform
passive-suggestion-terraform-config-query = Помоги мне создать конфигурацию Terraform. Спроси у меня все необходимые подробности.
passive-suggestion-prisma-setup-label = Помоги мне настроить Prisma в этом проекте
passive-suggestion-prisma-setup-query = Помоги мне настроить Prisma в этом проекте.
passive-suggestion-install-dependencies-query = Помоги мне установить зависимости для {1}.
passive-suggestion-ruby-project-label = Помоги мне настроить новый проект на Ruby
passive-suggestion-ruby-project-query = Помоги мне настроить новый проект на Ruby. Спроси у меня все необходимые подробности.
passive-suggestion-modelfile-query = Помоги мне настроить Modelfile для {1}.
passive-suggestion-kubernetes-utilization-query = Помоги мне разобраться с использованием ресурсов в моем кластере.
passive-suggestion-kubernetes-inspect-query = Помоги мне изучить ресурсы Kubernetes.
passive-suggestion-docker-containers-query = Помоги мне управлять запущенными контейнерами.
passive-suggestion-docker-images-query = Помоги мне управлять образами Docker.
passive-suggestion-docker-compose-label = Помоги мне управлять {1} или устранять неполадки с помощью Docker Compose
passive-suggestion-docker-compose-query = Помоги мне управлять {1} или устранять неполадки с помощью Docker Compose.
passive-suggestion-docker-network-query = Помоги мне настроить контейнеры на использование {1}.
passive-suggestion-vagrant-box-query = Помоги мне настроить или доработать Vagrant box {1}.
passive-suggestion-vagrant-up-query = Помоги мне подготовить окружение или устранить проблемы с запуском Vagrant.
passive-suggestion-grep-search-query = Помоги мне найти {1} в коде по всем файлам.
passive-suggestion-find-search-query = Помоги мне искать код по файлам с помощью {1}.
passive-suggestion-ssh-keygen-query = Помоги мне пошагово сгенерировать SSH-ключ.

# =============================================================================
# SECTION: remaining-ui-surfaces (Owner: agent-i18n-remaining)
# Files: app/src/workspace, app/src/terminal, app/src/code, app/src/notebooks,
#        app/src/ai, app/src/settings_view, app/src/workflows, app/src/view_components
# =============================================================================

common-update = Обновить
common-reject = Отклонить
common-open-link = Открыть ссылку
common-open-file = Открыть файл
common-open-folder = Открыть папку
common-name = Имя
common-rule = Правило
common-skip-for-now = Пропустить пока
common-never = Никогда
common-save-changes = Сохранить изменения
common-do-not-show-again = Больше не показывать
common-dont-show-again-with-period = Больше не показывать.
common-refresh = Обновить
common-resource-not-found-or-access-denied = Ресурс не найден или доступ запрещен
workspace-close-session = Закрыть сессию
workspace-auto-reload = Автоперезагрузка
workspace-add-new-repo = {" "}+ Добавить репозиторий
workspace-notification-permission-denied-toast = У Zap нет разрешения отправлять уведомления на рабочем столе.
workspace-troubleshoot-notifications-link = Устранение неполадок с уведомлениями
workspace-plan-synced-to-warp-drive-toast = План синхронизирован с вашим Zap Drive
workspace-remote-control-link-copied-toast = Ссылка для удаленного управления скопирована.
workspace-update-now = Обновить сейчас
workspace-update-warp = Обновить Zap
workspace-app-out-of-date-needs-update = Ваше приложение устарело и нуждается в обновлении.
workspace-restart-app-and-update-now = Перезапустить приложение и обновить сейчас
workspace-sampling-process-toast = Сбор данных о процессе в течение 3 секунд…
workspace-version-deprecation-banner = Ваше приложение устарело, и некоторые функции могут работать некорректно. Пожалуйста, обновитесь немедленно.
workspace-version-deprecation-without-permissions-banner = Если не обновиться немедленно, некоторые функции Zap могут работать некорректно, но Zap не может выполнить обновление.
workspace-new-version-unable-to-update-banner = Доступна новая версия, но Zap не может выполнить обновление.
workspace-unable-to-launch-new-installed-version = Zap не удалось запустить новую установленную версию.
tab-config-session-type = Тип сессии
terminal-copy-error = Копировать ошибку
terminal-authenticate-with-github = Авторизоваться через GitHub
terminal-create-environment = Создать окружение
terminal-regenerate-agents-file = Пересоздать файл AGENTS.md
terminal-view-index-status = Просмотреть статус индексации
terminal-shared-session-request-edit-access = Запросить доступ на редактирование
terminal-create-team = Создать команду
terminal-warpify-without-tmux = Warpify без TMUX
terminal-continue-without-warpification = Продолжить без Warpification
terminal-always-install = Всегда устанавливать
terminal-never-install = Никогда не устанавливать
terminal-ssh-report-issue-prefix = Мы активно работаем над улучшением стабильности SSH в Zap. Пожалуйста, рассмотрите возможность{" "}
terminal-ssh-report-issue-link = создать issue
terminal-ssh-report-issue-suffix = {" "}на GitHub, чтобы мы могли лучше определить проблему.
terminal-ssh-why-need-tmux = Зачем мне нужен tmux?
terminal-ssh-file-uploads-title = Загрузка файлов
terminal-ssh-close-upload-session = Закрыть сессию загрузки
terminal-ssh-view-upload-session = Просмотреть сессию загрузки
terminal-reveal-secret = Показать секрет
terminal-hide-secret = Скрыть секрет
terminal-copy-secret = Копировать секрет
terminal-tag-agent-for-assistance = Отметить агента для помощи
terminal-save-as-workflow-secrets-tooltip = Блоки, содержащие секреты, нельзя сохранить.
terminal-agent-mode-setup-title = Оптимизировать Zap для этой кодовой базы?
terminal-agent-mode-setup-description = Получите более умные и последовательные ответы, позволив агенту понять вашу кодовую базу и сгенерировать правила для нее. Вы также можете сделать это в любой момент, запустив /init
terminal-agent-mode-setup-optimize = Оптимизировать
terminal-no-active-conversation-to-export = Нет активного разговора для экспорта
terminal-slow-shell-startup-banner-prefix = Похоже, ваша shell запускается довольно долго…{"  "}
terminal-more-info = Подробнее
terminal-show-initialization-block = Показать блок инициализации
terminal-shell-process-exited = Процесс shell завершен
terminal-shell-process-could-not-start = Не удалось запустить процесс shell!
terminal-shell-process-exited-prematurely = Процесс shell завершился преждевременно!
terminal-shell-premature-subtext = Что-то пошло не так при запуске { $shell_detail } и его Warpify, из-за чего процесс завершился. Здесь отображается вывод скрипта Warpify, который может указывать на причину.
terminal-file-issue = Создать issue
notifications-banner-troubleshoot = Устранить неполадки
notifications-banner-dismissed-title = Мы больше не будем показывать этот баннер, но вы всегда можете перейти в Настройки, чтобы включить уведомления.
notifications-banner-disabled-title = Уведомления были выключены, но вы всегда можете перейти в Настройки, чтобы включить уведомления.
notifications-banner-enable = Включить
notifications-banner-permissions-accepted-title = Готово! Теперь вы можете получать уведомления на рабочем столе.
notifications-banner-permissions-denied-title = Zap не получил разрешение отправлять вам уведомления.
notifications-banner-permissions-error-title = Что-то пошло не так при запросе разрешений.
notifications-banner-allow-permissions-title = Не забудьте нажать «Разрешить» в запросе разрешений, чтобы завершить настройку уведомлений.
notifications-banner-configure-notifications = Настроить уведомления
notifications-banner-set-permissions = Настроить разрешения
ai-edit-api-keys = Редактировать API-ключи
ai-block-manage-agent-permissions = Управлять разрешениями агента
agent-zero-state-visit-docs = Открыть документацию
ai-execution-profile-agent-decides = Решает агент
ai-execution-profile-always-ask = Всегда спрашивать
ai-execution-profile-ask-on-first-write = Спрашивать при первой записи
ai-execution-profile-never-ask = Никогда не спрашивать
ai-execution-profile-ask-unless-auto-approve = Спрашивать всегда, кроме автоодобрения
code-accept-and-save = Принять и сохранить
code-hunk-label = Фрагмент:
code-discard-this-version = Отменить эту версию
code-overwrite = Перезаписать
code-review-send-to-agent = Отправить агенту
code-review-open-pr = Открыть PR
code-review-pr-created-toast = PR успешно создан.
code-review-comments-sent-to-agent = Комментарии отправлены агенту
code-review-could-not-submit-comments = Не удалось отправить комментарии агенту
code-review-tooltip-view-changes = Просмотреть изменения
code-review-diffs-local-workspaces-only = Диффы работают только для локальных рабочих пространств.
code-review-diffs-git-repositories-only = Диффы работают только для git-репозиториев.
code-review-diffs-wsl-unsupported = Диффы пока не работают в WSL.
code-review-generating-commit-message-placeholder = Генерация сообщения коммита…
code-review-type-commit-message-placeholder = Введите сообщение коммита
code-review-committing-loading = Создание коммита…
code-review-commit-message-label = Сообщение коммита
code-review-no-non-outdated-comments-to-send = Нет неустаревших комментариев для отправки
code-review-send-diff-comments-to = Отправить комментарии к диффу: { $label }
code-review-ai-must-be-enabled-to-send-comments = Чтобы отправлять комментарии агенту, необходимо включить AI
code-review-agent-code-review-requires-ai-credits = Для Code Review агентом необходимы AI-кредиты
code-review-all-terminals-are-busy = Все терминалы заняты
code-review-send-diff-comments-to-agent = Отправить комментарии к диффу агенту
code-failed-to-load-file-toast = Не удалось загрузить файл.
code-failed-to-save-file-toast = Не удалось сохранить файл.
code-file-saved-toast = Файл сохранен.
notebook-apply-link = Применить ссылку
notebook-sync-conflict-resolution-message = Не удалось сохранить этот блокнот, так как во время вашего редактирования в него были внесены изменения. Скопируйте свою работу и обновите блокнот.
notebook-sync-feature-not-available-message = Не удалось сохранить этот блокнот на сервере, так как функция временно недоступна. Изменения сохранены локально. Повторите попытку позже.
notebook-link-copied-toast = Ссылка скопирована
settings-share-with-team = Сохранить локально
tooltip-secrets-not-sent-to-warp-server = *Секреты не отправляются на сервер Zap.
editor-voice-limit-hit-toast = Вы достигли лимита голосовых запросов. Ваш лимит будет обновлен в рамках следующего цикла.
editor-voice-error-toast = Произошла ошибка при обработке голосового ввода.
ai-copied-branch-name-toast = Имя ветки скопировано
workflow-new-enum = Новый enum
workflow-edit-enum = Редактировать enum
workflow-enum-variant-placeholder = Вариант
workflow-enum-variants = Варианты
quit-warning-dont-save = Не сохранять
quit-warning-show-running-processes = Показать запущенные процессы
quit-warning-save-changes-title = Сохранить изменения?
