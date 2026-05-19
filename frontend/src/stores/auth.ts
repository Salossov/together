import { defineStore } from 'pinia'
import { Auth, Profile, getToken, setToken, type AuthResp, type UserMe } from '@/api'

interface State {
  token: string | null
  username: string | null
  internalUuid: string | null
  authType: 'LOCAL' | 'MICROSOFT' | null
  me: UserMe | null
  loading: boolean
}

export const useAuth = defineStore('auth', {
  state: (): State => ({
    token: null,
    username: null,
    internalUuid: null,
    authType: null,
    me: null,
    loading: false,
  }),
  getters: {
    isAuthed: (s) => !!s.token,
  },
  actions: {
    apply(r: AuthResp) {
      this.token = r.token
      this.username = r.username
      this.internalUuid = r.internal_uuid
      this.authType = r.auth_type
      setToken(r.token)
      try { localStorage.setItem('username', r.username) } catch {}
    },

    async restore() {
      const t = getToken()
      if (!t) return
      this.token = t
      this.username = localStorage.getItem('username')
      // Лёгкая проверка: попросим бэк подтвердить токен
      try {
        const r = await Auth.verify(t)
        if (r.status === 'valid') {
          this.username = r.username ?? this.username
          this.internalUuid = r.internal_uuid ?? this.internalUuid
        } else {
          this.logout()
        }
      } catch {
        // оффлайн — оставим токен, юзер сам разлогинится если что
      }
    },

    async login(username: string, password: string) {
      this.loading = true
      try {
        const r = await Auth.login(username, password)
        this.apply(r)
      } finally { this.loading = false }
    },

    async register(username: string, email: string | null, password: string) {
      this.loading = true
      try {
        const r = await Auth.register(username, email, password)
        this.apply(r)
      } finally { this.loading = false }
    },

    async finishMicrosoft(code: string) {
      this.loading = true
      try {
        const r = await Auth.microsoft(code)
        this.apply(r)
      } finally { this.loading = false }
    },

    async loadMe() {
      this.me = await Profile.me()
    },

    logout() {
      this.token = null
      this.username = null
      this.internalUuid = null
      this.authType = null
      this.me = null
      setToken(null)
      try { localStorage.removeItem('username') } catch {}
    },
  },
})
