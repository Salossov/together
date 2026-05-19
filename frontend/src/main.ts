import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './router'
import { useAuth } from './stores/auth'
import './style.css'

const app = createApp(App)
app.use(createPinia())

// Восстановить токен из localStorage до маунта
useAuth().restore()

app.use(router)
app.mount('#app')
