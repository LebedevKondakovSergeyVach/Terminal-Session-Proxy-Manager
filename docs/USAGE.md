# 📖 Справочник Команд

## 1. Проверка статуса (`status`)
```bash
proxy status         # Вывод информации о сети
proxy status --json  # Вывод в формате JSON
```

---

## 2. Управление профилями (`profile`)
```bash
proxy-cli profile list       # Список всех профилей
proxy use v2ray              # Переключить активный профиль
proxy-cli profile set myvps --port 1080 --host 127.0.0.1  # Добавить/обновить профиль
proxy-cli profile remove myvps # Удалить профиль
```

---

## 3. Замер задержки (`ping`)
```bash
proxy ping                 # Замер задержки до сервисов
proxy-cli ping --timeout 2000 # Пинг с таймаутом 2000мс
```

---

## 4. Диагностика (`diagnose`)
```bash
proxy diagnose # Проверка TCP сокета, переменных окружения и API
```

---

## 5. Выполнение разовых команд (`run`)
```bash
proxy run -- curl https://api.openai.com
```

---

## 6. Автодополнение (`completions`)
```bash
proxy-cli completions zsh > ~/.zsh/completions/_proxy-cli
```
