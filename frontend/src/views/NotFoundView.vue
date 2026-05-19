<script setup lang="ts">
import { RouterLink } from 'vue-router'
</script>

<template>
  <div class="text-center py-16">
    <p class="text-6xl font-bold text-brand">404</p>
    <p class="mt-2 text-zinc-400">Такой страницы нет.</p>
    <RouterLink :to="{ name: 'home' }" class="btn-primary mt-6 inline-flex">На главную</RouterLink>
  </div>
</template>
