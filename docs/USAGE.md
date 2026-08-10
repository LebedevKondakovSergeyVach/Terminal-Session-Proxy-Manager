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

## 3. Управление профилями (`profile`)
```bash
proxy-cli profile list       # Список всех профилей
proxy switch                 # Интерактивное меню выбора профиля
proxy use v2ray              # Переключить активный профиль
proxy best                   # Автовыбор самого быстрого прокси
proxy benchmark              # Измерение пинга и доступности профилей
proxy import <file/url>      # Импорт профилей из JSON/URL
proxy-cli profile set myvps --port 1080 --host 127.0.0.1  # Добавить/обновить профиль
proxy-cli profile remove myvps # Удалить профиль
```

---

## 4. Замер задержки (`ping`)
```bash
proxy ping                 # Замер задержки до сервисов
proxy-cli ping --timeout 2000 # Пинг с таймаутом 2000мс
```

---

## 5. Диагностика (`diagnose`)
```bash
proxy diagnose # Проверка TCP сокета, переменных окружения и API
```

---

## 6. Выполнение разовых команд (`run`)
```bash
proxy run -- curl https://api.openai.com
```

---

## 7. Автодополнение (`completions`)
```bash
proxy-cli completions zsh > ~/.zsh/completions/_proxy-cli
```
