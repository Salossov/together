# together-frontend

Vue 3 + Vite + TypeScript + Pinia + TailwindCSS.

Стек: Vue 3.5 + Vite 8 + vue-router 5 + Pinia 3 + TailwindCSS 4 (через `@tailwindcss/vite`) + TypeScript 6.

## Запуск

```bash
cd frontend
cp .env.example .env
bun install
bun run dev      # http://127.0.0.1:5173
```

Бэкенд должен крутиться на `VITE_API_BASE` (по умолчанию `http://127.0.0.1:8080`)
и иметь этот origin в `CORS_ORIGINS`.

## Страницы

| Path | Назначение |
|---|---|
| `/` | Главная: статус бэка, краткое описание |
| `/register` | Регистрация локального аккаунта |
| `/login` | Вход + кнопка Microsoft (если задан `VITE_MS_AUTHORIZE_URL`) |
| `/profile` | Профиль: данные, JWT, загрузка скина |
| `/setup` | Инструкция как зайти на сервер + копирование JWT |
| `/auth/microsoft/callback` | Принимает `?code=` от Microsoft, обменивает на JWT |

## Сборка

```bash
bun run build    # → dist/
bun run preview
```
