<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, RouterLink } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { apiError } from '@/api'

const auth = useAuth()
const router = useRouter()

const username = ref('')
const email = ref('')
const password = ref('')
const password2 = ref('')
const error = ref<string | null>(null)

const usernameOk = computed(() => /^[A-Za-z0-9_]{3,16}$/.test(username.value))
const passwordsOk = computed(() => password.value === password2.value)

async function submit() {
  error.value = null
  if (!usernameOk.value) {
    error.value = 'Ник: 3–16 символов, только латиница, цифры и _'
    return
  }
  if (password.value.length < 6) { error.value = 'Минимум 6 символов в пароле'; return }
  if (!passwordsOk.value)        { error.value = 'Пароли не совпадают'; return }

  try {
    await auth.register(username.value.trim(), email.value.trim() || null, password.value)
    router.push({ name: 'profile' })
  } catch (e) {
    error.value = apiError(e)
  }
}
</script>

<template>
  <div class="max-w-md mx-auto">
    <div class="card">
      <h1 class="text-2xl font-semibold mb-1">Регистрация</h1>
      <p class="text-sm text-zinc-400 mb-4">Локальный аккаунт без проверки лицензии.</p>

      <form @submit.prevent="submit" class="space-y-3">
        <div>
          <label class="label">Ник в игре</label>
          <input v-model="username" class="input" required minlength="3" maxlength="16" />
          <p v-if="username && !usernameOk" class="field-error">3–16 символов: A-Z, a-z, 0-9, _</p>
        </div>
        <div>
          <label class="label">Email <span class="text-zinc-500">(опционально)</span></label>
          <input v-model="email" type="email" class="input" autocomplete="email" />
        </div>
        <div>
          <label class="label">Пароль</label>
          <input v-model="password" type="password" class="input" autocomplete="new-password" required minlength="6" />
        </div>
        <div>
          <label class="label">Повторите пароль</label>
          <input v-model="password2" type="password" class="input" autocomplete="new-password" required minlength="6" />
        </div>

        <p v-if="error" class="field-error">{{ error }}</p>

        <button class="btn-primary w-full" :disabled="auth.loading">
          {{ auth.loading ? 'Создаём…' : 'Создать аккаунт' }}
        </button>
      </form>

      <p class="text-sm text-zinc-400 mt-4 text-center">
        Уже есть аккаунт? <RouterLink :to="{ name: 'login' }">Войти</RouterLink>
      </p>
    </div>
  </div>
</template>
