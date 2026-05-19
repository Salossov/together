<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter, useRoute, RouterLink } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { apiError, Auth } from '@/api'

const auth = useAuth()
const router = useRouter()
const route = useRoute()

const username = ref('')
const password = ref('')
const error = ref<string | null>(null)

interface MsCfg {
  url: string
  configured: boolean
  mode: 'redirect' | 'manual_paste'
  redirect_uri?: string
}
const msCfg = ref<MsCfg | null>(null)
const msLoading = ref(false)

// Manual-paste flow state
const showPaste = ref(false)
const pasteValue = ref('')
const pasteError = ref<string | null>(null)
const pasting = ref(false)

onMounted(async () => {
  try {
    msCfg.value = await Auth.microsoftUrl()
  } catch {
    msCfg.value = null
  }
})

async function submit() {
  error.value = null
  try {
    await auth.login(username.value.trim(), password.value)
    const next = (route.query.next as string) || '/profile'
    router.push(next)
  } catch (e) {
    error.value = apiError(e)
  }
}

function microsoftLogin() {
  if (!msCfg.value) return
  if (msCfg.value.mode === 'redirect') {
    window.location.href = msCfg.value.url
  } else {
    // launcher-режим: открываем MS в новой вкладке, после логина юзер
    // вставит URL пустой страницы login.live.com обратно сюда
    window.open(msCfg.value.url, '_blank', 'noopener')
    showPaste.value = true
    pasteError.value = null
  }
}

/** Достаёт `?code=` или `&code=` из произвольной строки (полный URL или фрагмент) */
function extractCode(input: string): string | null {
  const trimmed = input.trim()
  if (!trimmed) return null
  // Если выглядит как URL — пробуем распарсить
  try {
    const u = new URL(trimmed)
    const c = u.searchParams.get('code')
    if (c) return c
  } catch { /* не URL — продолжим */ }
  // Иначе ищем code= в любом виде
  const m = trimmed.match(/[?&]?code=([^&\s]+)/)
  return m ? decodeURIComponent(m[1]) : null
}

async function submitPaste() {
  pasteError.value = null
  const code = extractCode(pasteValue.value)
  if (!code) {
    pasteError.value = 'Не нашёл code= в этой строке. Скопируй URL целиком из адресной строки пустой страницы.'
    return
  }
  pasting.value = true
  try {
    await auth.finishMicrosoft(code)
    router.push({ name: 'profile' })
  } catch (e) {
    pasteError.value = apiError(e)
  } finally {
    pasting.value = false
  }
}
</script>

<template>
  <div class="max-w-md mx-auto">
    <div class="card">
      <h1 class="text-2xl font-semibold mb-4">Вход</h1>

      <form @submit.prevent="submit" class="space-y-3">
        <div>
          <label class="label">Ник</label>
          <input v-model="username" class="input" autocomplete="username" required minlength="3" maxlength="16" />
        </div>
        <div>
          <label class="label">Пароль</label>
          <input v-model="password" class="input" type="password" autocomplete="current-password" required minlength="6" />
        </div>

        <p v-if="error" class="field-error">{{ error }}</p>

        <button class="btn-primary w-full" :disabled="auth.loading">
          {{ auth.loading ? 'Входим…' : 'Войти' }}
        </button>
      </form>

      <div v-if="msCfg?.configured" class="mt-4">
        <div class="text-center text-xs text-zinc-500 my-3">или</div>
        <button class="btn-ghost w-full" :disabled="msLoading" @click="microsoftLogin">
          <svg viewBox="0 0 23 23" class="w-4 h-4">
            <path fill="#f25022" d="M1 1h10v10H1z"/>
            <path fill="#7fba00" d="M12 1h10v10H12z"/>
            <path fill="#00a4ef" d="M1 12h10v10H1z"/>
            <path fill="#ffb900" d="M12 12h10v10H12z"/>
          </svg>
          Войти через Microsoft
        </button>

        <p v-if="msCfg.mode === 'manual_paste'" class="text-[11px] text-zinc-500 text-center mt-2">
          Launcher-режим: после входа Microsoft покажет пустую страницу — скопируй её URL обратно сюда.
        </p>
      </div>

      <p class="text-sm text-zinc-400 mt-4 text-center">
        Нет аккаунта? <RouterLink :to="{ name: 'register' }">Зарегистрируйтесь</RouterLink>
      </p>
    </div>

    <!-- Modal: manual paste -->
    <div v-if="showPaste"
         class="fixed inset-0 bg-black/70 backdrop-blur-sm grid place-items-center p-4 z-20"
         @click.self="showPaste = false">
      <div class="card max-w-lg w-full">
        <h2 class="text-xl font-semibold mb-3">Завершите вход через Microsoft</h2>
        <ol class="text-sm text-zinc-300 space-y-2 list-decimal pl-5 mb-4">
          <li>В открывшейся вкладке войдите в свой Microsoft-аккаунт.</li>
          <li>После входа Microsoft перебросит вас на <b>пустую страницу</b>
              <code class="text-xs">login.live.com/oauth20_desktop.srf?code=…</code>.</li>
          <li>Скопируйте URL этой пустой страницы целиком из адресной строки и вставьте сюда:</li>
        </ol>

        <textarea v-model="pasteValue"
                  class="input text-xs h-24"
                  placeholder="https://login.live.com/oauth20_desktop.srf?code=M.C..."></textarea>
        <p v-if="pasteError" class="field-error">{{ pasteError }}</p>

        <div class="flex gap-2 mt-4">
          <button class="btn-primary flex-1" :disabled="pasting" @click="submitPaste">
            {{ pasting ? 'Проверяем…' : 'Войти' }}
          </button>
          <button class="btn-ghost" @click="showPaste = false">Отмена</button>
        </div>

        <p class="text-[11px] text-zinc-500 mt-3">
          Нужен только параметр <code>?code=…</code> — мы его сами вытащим, можно вставлять весь URL.
        </p>
      </div>
    </div>
  </div>
</template>
