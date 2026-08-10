# 🐚 Интеграция с Оболочками

## Zsh (`~/.zshrc`)
```zsh
eval "$(proxy-cli init zsh)"
```
> Совместимо с Powerlevel10k Instant Prompt (не создает консольного вывода при старте).

## Bash (`~/.bashrc`)
```bash
eval "$(proxy-cli init bash)"
```

---

## Доступные функции

| Команда | Описание |
| :--- | :--- |
| `proxy on` | Включить прокси для текущей сессии |
| `proxy off` | Выключить прокси |
| `proxy toggle` | Переключить состояние прокси |
| `proxy status` | Проверить статус сети |
| `proxy use <key>` | Сменить активный профиль |
| `proxy ping` | Замер задержек до сервисов |
| `proxy diagnose` | Диагностика сокета и API |
| `proxy run -- <cmd>` | Выполнить команду через прокси |
| `prompt_proxy_status` | Вывести статус для сегмента Prompt |
