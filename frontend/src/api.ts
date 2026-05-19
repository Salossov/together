import axios, { AxiosError } from 'axios'

export const API_BASE: string =
  (import.meta.env.VITE_API_BASE as string | undefined)?.replace(/\/+$/, '') ||
  'http://127.0.0.1:8080'

export const api = axios.create({
  baseURL: API_BASE,
  timeout: 20_000,
})

// JWT interceptor
let bearer: string | null = null
export function setToken(t: string | null) {
  bearer = t
  if (t) localStorage.setItem('jwt', t)
  else localStorage.removeItem('jwt')
}
export function getToken(): string | null {
  return bearer ?? localStorage.getItem('jwt')
}

api.interceptors.request.use((cfg) => {
  const t = getToken()
  if (t) cfg.headers.Authorization = `Bearer ${t}`
  return cfg
})

// ---- Типы API ----

export interface AuthResp {
  token: string
  username: string
  internal_uuid: string
  auth_type: 'LOCAL' | 'MICROSOFT'
}

export interface UserMe {
  id: number
  username: string
  email: string | null
  internal_uuid: string
  skin_base64: string | null
  skin_signature: string | null
  auth_type: 'LOCAL' | 'MICROSOFT'
}

export interface VerifyTokenResp {
  status: 'valid' | 'invalid'
  username?: string
  internal_uuid?: string
}

export interface UpdateFile {
  path: string
  sha256: string
  url: string
  size: number
}
export interface VersionCheckResp {
  update: UpdateFile[]
  remove: string[]
}

// ---- Хелперы ----

export function apiError(e: unknown): string {
  if (e instanceof AxiosError) {
    const data = e.response?.data as { error?: string } | undefined
    if (data?.error) return data.error
    if (e.response?.status) return `HTTP ${e.response.status}`
    return e.message
  }
  return String(e)
}

// ---- Эндпоинты ----

export const Auth = {
  register: (username: string, email: string | null, password: string) =>
    api.post<AuthResp>('/api/auth/register', { username, email, password }).then(r => r.data),
  login: (username: string, password: string) =>
    api.post<AuthResp>('/api/auth/login', { username, password }).then(r => r.data),
  microsoft: (code: string) =>
    api.post<AuthResp>('/api/auth/microsoft', { code }).then(r => r.data),
  microsoftUrl: () =>
    api.get<{
      url: string
      configured: boolean
      mode: 'redirect' | 'manual_paste'
      redirect_uri?: string
    }>('/api/auth/microsoft/url').then(r => r.data),
  verify: (token: string) =>
    api.post<VerifyTokenResp>('/api/launcher/verify_token', { token }).then(r => r.data),
}

export const Profile = {
  me: () => api.get<UserMe>('/api/profile/me').then(r => r.data),
  uploadSkin: (file: File) => {
    const fd = new FormData()
    fd.append('file', file)
    return api.post<{ ok: boolean; signed: boolean; size: number }>(
      '/api/profile/skin', fd, { headers: { 'Content-Type': 'multipart/form-data' } },
    ).then(r => r.data)
  },
}

export const Launcher = {
  versionCheck: (files: Record<string, string>) =>
    api.get<VersionCheckResp>('/api/launcher/version_check', { data: { files } })
       .then(r => r.data),
}
