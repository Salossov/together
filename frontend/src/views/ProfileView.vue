<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useAuth } from '@/stores/auth'
import { apiError, Profile } from '@/api'
import SkinUploader from '@/components/SkinUploader.vue'

const auth = useAuth()
const error = ref<string | null>(null)
const loading = ref(true)

async function load() {
  loading.value = true
  error.value = null
  try { await auth.loadMe() } catch (e) { error.value = apiError(e) }
  finally { loading.value = false }
}
onMounted(load)

async function onSkinUploaded() {
  // обновим, чтобы получить новую base64 для превью
  await load()
}

const token = ref<string | null>(null)
function showToken() {
  token.value = auth.token
}
function copyToken() {
  if (auth.token) navigator.clipboard?.writeText(auth.token)
}
</script>

<template>
  <div class="grid md:grid-cols-3 gap-6">
    <!-- Карточка профиля -->
    <div class="card md:col-span-1">
      <h2 class="font-semibold mb-3">Профиль</h2>

      <p v-if="loading" class="text-zinc-400">Загрузка…</p>
      <p v-else-if="error" class="field-error">{{ error }}</p>

      <template v-else-if="auth.me">
        <div class="space-y-2 text-sm">
          <div>
            <span class="text-zinc-500">Ник:</span>
            <span class="ml-2 text-zinc-100 font-medium">{{ auth.me.username }}</span>
          </div>
          <div>
            <span class="text-zinc-500">Тип:</span>
            <span class="ml-2 inline-block px-2 py-0.5 rounded text-xs"
                  :class="auth.me.auth_type === 'MICROSOFT' ? 'bg-blue-900 text-blue-300' : 'bg-zinc-800 text-zinc-300'">
              {{ auth.me.auth_type }}
            </span>
          </div>
          <div v-if="auth.me.email">
            <span class="text-zinc-500">Email:</span>
            <span class="ml-2 text-zinc-300">{{ auth.me.email }}</span>
          </div>
          <div>
            <span class="text-zinc-500">UUID:</span>
            <code class="ml-2 text-xs text-zinc-300 break-all">{{ auth.me.internal_uuid }}</code>
          </div>
        </div>

        <hr class="border-zinc-800 my-4" />

        <button class="btn-ghost w-full text-sm" @click="showToken">Показать JWT</button>
        <div v-if="token" class="mt-3">
          <textarea readonly class="input text-xs h-24 break-all">{{ token }}</textarea>
          <button class="btn-ghost w-full mt-2 text-xs" @click="copyToken">Скопировать</button>
          <p class="text-xs text-zinc-500 mt-2">Этот токен использует ваш лаунчер. Не показывайте посторонним.</p>
        </div>
      </template>
    </div>

    <!-- Скин -->
    <div class="card md:col-span-2">
      <h2 class="font-semibold mb-3">Скин</h2>
      <SkinUploader :current="auth.me?.skin_base64 ?? null" @uploaded="onSkinUploaded" />
    </div>
  </div>
</template>
