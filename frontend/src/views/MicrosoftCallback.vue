<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { apiError } from '@/api'

const auth = useAuth()
const router = useRouter()
const route = useRoute()

const status = ref<'pending' | 'ok' | 'fail'>('pending')
const error = ref<string | null>(null)

onMounted(async () => {
  const code = route.query.code as string | undefined
  const err = route.query.error as string | undefined
  if (err) { status.value = 'fail'; error.value = String(err); return }
  if (!code) { status.value = 'fail'; error.value = 'нет ?code в URL'; return }

  try {
    await auth.finishMicrosoft(code)
    status.value = 'ok'
    setTimeout(() => router.replace({ name: 'profile' }), 400)
  } catch (e) {
    status.value = 'fail'
    error.value = apiError(e)
  }
})
</script>

<template>
  <div class="max-w-md mx-auto card text-center">
    <h1 class="text-xl font-semibold mb-3">Microsoft sign-in</h1>

    <p v-if="status === 'pending'" class="text-zinc-400">Проверяем код у Microsoft и Mojang…</p>
    <p v-else-if="status === 'ok'"  class="text-brand">Готово, переходим в профиль…</p>
    <div v-else>
      <p class="field-error mb-3">{{ error }}</p>
      <router-link :to="{ name: 'login' }" class="btn-ghost">Назад ко входу</router-link>
    </div>
  </div>
</template>
