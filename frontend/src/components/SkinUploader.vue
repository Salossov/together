<script setup lang="ts">
import { computed, ref } from 'vue'
import { apiError, Profile } from '@/api'

const props = defineProps<{ current: string | null }>()
const emit = defineEmits<{ (e: 'uploaded'): void }>()

const fileInput = ref<HTMLInputElement | null>(null)
const preview = ref<string | null>(null)
const file = ref<File | null>(null)
const error = ref<string | null>(null)
const loading = ref(false)
const lastResult = ref<{ signed: boolean; size: number } | null>(null)

const currentSrc = computed(() => {
  if (preview.value) return preview.value
  if (props.current) return `data:image/png;base64,${props.current}`
  return null
})

function pick() { fileInput.value?.click() }

function onFile(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0]
  if (!f) return
  if (f.type !== 'image/png') { error.value = 'Нужен PNG'; return }
  error.value = null
  file.value = f
  preview.value = URL.createObjectURL(f)
}

async function upload() {
  if (!file.value) return
  error.value = null
  loading.value = true
  try {
    lastResult.value = await Profile.uploadSkin(file.value)
    file.value = null
    if (preview.value) { URL.revokeObjectURL(preview.value); preview.value = null }
    emit('uploaded')
  } catch (e) {
    error.value = apiError(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="flex flex-col md:flex-row gap-6">
    <!-- Превью -->
    <div class="flex flex-col items-center">
      <div class="w-32 h-64 bg-zinc-900 border border-zinc-800 rounded-lg grid place-items-center overflow-hidden">
        <img v-if="currentSrc"
             :src="currentSrc"
             class="image-pixel w-full h-auto"
             alt="skin" />
        <span v-else class="text-zinc-600 text-sm text-center px-2">Скин не загружен</span>
      </div>
      <p class="text-xs text-zinc-500 mt-2">64×64 / 64×32 PNG</p>
    </div>

    <!-- Управление -->
    <div class="flex-1 space-y-3">
      <input ref="fileInput" type="file" accept="image/png" class="hidden" @change="onFile" />

      <div class="flex gap-2 flex-wrap">
        <button class="btn-ghost" @click="pick">Выбрать файл…</button>
        <button class="btn-primary" :disabled="!file || loading" @click="upload">
          {{ loading ? 'Загружаем…' : 'Загрузить' }}
        </button>
      </div>

      <p v-if="file" class="text-sm text-zinc-400">{{ file.name }} · {{ (file.size/1024).toFixed(1) }} КБ</p>
      <p v-if="error" class="field-error">{{ error }}</p>

      <div v-if="lastResult" class="text-sm rounded-lg p-3"
           :class="lastResult.signed ? 'bg-emerald-900/40 text-emerald-300' : 'bg-amber-900/40 text-amber-300'">
        Скин сохранён ({{ (lastResult.size/1024).toFixed(1) }} КБ).
        <span v-if="lastResult.signed">Получена подпись Mojang — ванильный клиент его примет.</span>
        <span v-else>Без подписи Mojang — ванильный клиент покажет дефолтный скин (нужен MINESKIN_API_KEY на бэке).</span>
      </div>

      <div class="text-xs text-zinc-500 leading-relaxed">
        Скин отдаётся серверу во время handshake. Если вы зашли через Microsoft —
        используется ваш официальный скин Mojang.
      </div>
    </div>
  </div>
</template>

<style scoped>
.image-pixel {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}
</style>
