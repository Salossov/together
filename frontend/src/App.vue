<script setup lang="ts">
import NavBar from '@/components/NavBar.vue'
</script>

<template>
  <div class="min-h-full flex flex-col">
    <NavBar />
    <main class="flex-1 container mx-auto max-w-5xl px-4 py-8">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
    <footer class="text-center text-xs text-zinc-500 py-4 border-t border-zinc-900">
      together · NeoForge 1.21.1 · {{ new Date().getFullYear() }}
    </footer>
  </div>
</template>

<style>
.fade-enter-active, .fade-leave-active { transition: opacity .15s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
