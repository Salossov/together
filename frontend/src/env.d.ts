/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

interface ImportMetaEnv {
  readonly VITE_API_BASE?: string
  readonly VITE_SERVER_NAME?: string
  readonly VITE_SERVER_HOST?: string
  readonly VITE_SERVER_PORT?: string

}
interface ImportMeta {
  readonly env: ImportMetaEnv
}
