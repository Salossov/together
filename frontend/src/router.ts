import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useAuth } from '@/stores/auth'

const routes: RouteRecordRaw[] = [
  { path: '/',          component: () => import('@/views/HomeView.vue'),     name: 'home' },
  { path: '/login',     component: () => import('@/views/LoginView.vue'),    name: 'login',    meta: { guest: true } },
  { path: '/register',  component: () => import('@/views/RegisterView.vue'), name: 'register', meta: { guest: true } },
  { path: '/profile',   component: () => import('@/views/ProfileView.vue'),  name: 'profile',  meta: { auth: true } },
  { path: '/setup',     component: () => import('@/views/SetupView.vue'),    name: 'setup' },
  { path: '/auth/microsoft/callback', component: () => import('@/views/MicrosoftCallback.vue'), name: 'ms-callback' },
  { path: '/:pathMatch(.*)*', component: () => import('@/views/NotFoundView.vue') },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to) => {
  const auth = useAuth()
  if (to.meta.auth && !auth.isAuthed) return { name: 'login', query: { next: to.fullPath } }
  if (to.meta.guest && auth.isAuthed) return { name: 'profile' }
})
