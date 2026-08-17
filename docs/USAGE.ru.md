# 📖 Справочник Команд

## 1. Проверка статуса (`status`)
```bash
proxy status         # Вывод информации о сети
proxy status --json  # Вывод в формате JSON
```

---

## 2. Переключение языка (`lang`)
```bash
proxy lang en # Переключить на английский язык
proxy lang ru # Переключить на русский язык
```

---

## 3. Управление прокси в Git (`git`)
```bash
proxy git on     # Включить прокси для git clone / git push
proxy git off    # Сбросить прокси в git config
proxy git status # Показать текущие настройки git config
```

---

## 4. Экспорт конфигураций (`export`)
```bash
proxy export docker  # Флаги --build-arg для Docker
proxy export curl    # Флаги -x для cURL
proxy export envfile # Содержимое .env файла
```

---

## 5. Замер скорости скачивания (`speedtest`)
```bash
proxy speedtest # Измерить реальную пропускную способность прокси в Мб/с
```

---

## 6. Мониторинг здоровья и авто-восстановление (`monitor`)
```bash
proxy monitor # Проверить соединение; при сбое автопереключить на резервный прокси
```

---

## 7. Управление профилями (`profile`)
```bash
terminal-session-proxy-manager profile list       # Список всех профилей
proxy switch                 # Интерактивное меню выбора профиля стрелочками
proxy use v2ray              # Переключить активный профиль
proxy dash                   # Запустить интерактивный TUI Дашборд
proxy best                   # Автовыбор самого быстрого прокси
proxy benchmark              # Измерение пинга и доступности профилей
proxy import <file/url>      # Импорт профилей из JSON/URL
```

---

## 8. Замер задержки (`ping`)
```bash
proxy ping                 # Замер задержки до сервисов
terminal-session-proxy-manager ping --timeout 2000 # Пинг с таймаутом 2000мс
```

---

## 9. Диагностика (`diagnose`)
```bash
proxy diagnose # Проверка TCP сокета, переменных окружения и API
```

---

## 10. Выполнение разовых команд (`run`)
```bash
proxy run -- curl https://api.openai.com
```

---

## 11. Автодополнение (`completions`)
```bash
terminal-session-proxy-manager completions zsh > ~/.zsh/completions/_terminal-session-proxy-manager
```
