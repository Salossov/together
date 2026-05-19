<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, API_BASE } from '@/api'
import { RouterLink } from 'vue-router'

const status = ref<'checking' | 'online' | 'offline'>('checking')

onMounted(async () => {
  try {
    await api.get('/health', { timeout: 4000 })
    status.value = 'online'
  } catch {
    status.value = 'offline'
  }
})

const serverName = (import.meta.env.VITE_SERVER_NAME as string) || 'Together'
</script>

<template>
  <section class="grid md:grid-cols-2 gap-8 items-center">
    <div>
      <h1 class="text-4xl md:text-5xl font-bold leading-tight">
        Сервер <span class="text-brand">{{ serverName }}</span><br>
        с модпаком Create Aeronautics
      </h1>
      <p class="mt-4 text-zinc-400 leading-relaxed">
        Закрытый сервер для своих. Войдите через лицензию Microsoft или
        зарегистрируйте локальный аккаунт — модифицированный клиент сам
        проверит ваш JWT во время handshake.
      </p>

      <div class="mt-6 flex gap-3 flex-wrap">
        <RouterLink :to="{ name: 'register' }" class="btn-primary">Создать аккаунт</RouterLink>
        <RouterLink :to="{ name: 'setup' }"    class="btn-ghost">Как подключиться</RouterLink>
      </div>

      <div class="mt-6 text-sm">
        <span class="text-zinc-500">Бэкенд:</span>
        <code class="ml-2 text-zinc-300">{{ API_BASE }}</code>
        <span class="ml-3 inline-flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full"
                :class="{
                  'bg-zinc-500 animate-pulse': status === 'checking',
                  'bg-brand': status === 'online',
                  'bg-red-500': status === 'offline',
                }"></span>
          <span class="text-zinc-400">
            {{ status === 'checking' ? 'проверяем…' : status === 'online' ? 'онлайн' : 'офлайн' }}
          </span>
        </span>
      </div>
    </div>

    <div class="card">
      <h2 class="font-semibold mb-3">Что внутри</h2>
      <ul class="space-y-2 text-sm text-zinc-300">
        <li>• NeoForge 1.21.1, ванильный клиент</li>
        <li>• Авторизация через сайт (JWT 30 дней)</li>
        <li>• Microsoft OAuth для проверки лицензии</li>
        <li>• Кастомные скины с подписью Mojang</li>
        <li>• Манифест обновлений модов через лаунчер</li>
      </ul>
    </div>
  </section>
</template>
