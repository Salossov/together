<script setup lang="ts">
import { RouterLink, useRouter } from 'vue-router'
import { useAuth } from '@/stores/auth'

const auth = useAuth()
const router = useRouter()

function logout() {
  auth.logout()
  router.push({ name: 'home' })
}
</script>

<template>
  <header class="border-b border-zinc-900 bg-zinc-950/80 backdrop-blur sticky top-0 z-10">
    <div class="container mx-auto max-w-5xl px-4 h-14 flex items-center justify-between">
      <RouterLink :to="{ name: 'home' }" class="flex items-center gap-2 font-semibold text-zinc-100 hover:text-brand">
        <span class="w-7 h-7 rounded-md bg-brand grid place-items-center text-zinc-950 font-bold">T</span>
        together
      </RouterLink>

      <nav class="flex items-center gap-1 text-sm">
        <RouterLink :to="{ name: 'home' }"  class="px-3 py-2 rounded-md hover:bg-zinc-900 text-zinc-300">Главная</RouterLink>
        <RouterLink :to="{ name: 'setup' }" class="px-3 py-2 rounded-md hover:bg-zinc-900 text-zinc-300">Как зайти</RouterLink>

        <template v-if="!auth.isAuthed">
          <RouterLink :to="{ name: 'login' }"    class="px-3 py-2 rounded-md hover:bg-zinc-900 text-zinc-300">Войти</RouterLink>
          <RouterLink :to="{ name: 'register' }" class="btn-primary !py-1.5 !px-3 text-sm">Регистрация</RouterLink>
        </template>

        <template v-else>
          <RouterLink :to="{ name: 'profile' }" class="px-3 py-2 rounded-md hover:bg-zinc-900 text-zinc-300">
            {{ auth.username }}
          </RouterLink>
          <button @click="logout" class="px-3 py-2 rounded-md hover:bg-zinc-900 text-zinc-400">Выйти</button>
        </template>
      </nav>
    </div>
  </header>
</template>
