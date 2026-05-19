<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAuth } from '@/stores/auth'
import { API_BASE } from '@/api'

const auth = useAuth()

const host = (import.meta.env.VITE_SERVER_HOST as string) || 'mc.example.com'
const port = (import.meta.env.VITE_SERVER_PORT as string) || '25565'
const name = (import.meta.env.VITE_SERVER_NAME as string) || 'Together'

const tokenVisible = ref(false)
const addrLine = computed(() => port === '25565' ? host : `${host}:${port}`)

function copy(text: string) {
  navigator.clipboard?.writeText(text)
}
</script>

<template>
  <article class="prose prose-invert max-w-none">
    <h1>Как подключиться</h1>

    <ol class="space-y-6 list-decimal pl-5">
      <li>
        <h3 class="font-semibold mt-0">Установите NeoForge 1.21.1</h3>
        <p class="text-zinc-400 text-sm">
          Скачайте лаунчер модпака или поставьте NeoForge вручную на любом ванильном лаунчере
          (Prism / MultiMC / официальный).
        </p>
      </li>

      <li>
        <h3 class="font-semibold">Положите модпак в <code>mods/</code></h3>
        <p class="text-zinc-400 text-sm">
          Версии файлов лаунчер сверит автоматически с серверным манифестом
          (<code>GET /api/launcher/version_check</code>).
        </p>
      </li>

      <li>
        <h3 class="font-semibold">Авторизуйтесь</h3>
        <p class="text-zinc-400 text-sm">
          <template v-if="auth.isAuthed">
            Вы уже вошли как <b class="text-brand">{{ auth.username }}</b>.
            Откройте лаунчер и нажмите «Войти» — он использует ваш токен с этого сайта.
          </template>
          <template v-else>
            <router-link :to="{ name: 'login' }">Войдите</router-link> или
            <router-link :to="{ name: 'register' }">зарегистрируйтесь</router-link>,
            чтобы получить JWT-токен.
          </template>
        </p>

        <div v-if="auth.isAuthed" class="card mt-3 not-prose">
          <div class="flex items-center justify-between">
            <span class="text-sm text-zinc-400">Ваш JWT (срок: 30 дней)</span>
            <button class="text-xs text-brand hover:underline" @click="tokenVisible = !tokenVisible">
              {{ tokenVisible ? 'скрыть' : 'показать' }}
            </button>
          </div>
          <template v-if="tokenVisible">
            <textarea readonly class="input text-xs h-24 mt-2 break-all">{{ auth.token }}</textarea>
            <button class="btn-ghost text-xs mt-2" @click="copy(auth.token || '')">Скопировать</button>
          </template>
        </div>
      </li>

      <li>
        <h3 class="font-semibold">Подключитесь к серверу</h3>
        <div class="flex items-center gap-2 not-prose">
          <code class="px-3 py-2 bg-zinc-900 rounded-lg border border-zinc-800 text-zinc-100">{{ addrLine }}</code>
          <button class="btn-ghost text-xs" @click="copy(addrLine)">Скопировать</button>
        </div>
        <p class="text-zinc-400 text-sm mt-2">
          Имя сервера в списке: <b>{{ name }}</b>. Во время логина клиентский мод отправит ваш JWT
          в <code>POST {{ API_BASE }}/api/mc-server/validate_handshake</code> — если токен валиден,
          вас впустят с вашим скином и постоянным UUID. Если нет — кикнет с сообщением об истёкшей сессии.
        </p>
      </li>
    </ol>

    <hr class="border-zinc-800" />

    <h2>Если что-то не работает</h2>
    <ul class="text-sm space-y-1">
      <li><b>«Сессия истекла»</b> — войдите снова на сайте и обновите токен в лаунчере.</li>
      <li><b>«Ник занят»</b> — другой игрок уже зарегистрировал этот ник. UUID персонажа не меняется при смене ника на сайте.</li>
      <li><b>Скин не отображается</b> — без интеграции Mineskin/Mojang ванильный клиент игнорирует кастомные текстуры. Войдите через Microsoft, чтобы получить подписанный скин Mojang.</li>
    </ul>
  </article>
</template>
