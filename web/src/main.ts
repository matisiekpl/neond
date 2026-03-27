import {createApp} from 'vue'
import './style.css'
import App from './App.vue'
import {router} from '@/router.ts'
import {createPinia} from 'pinia'
import 'vue-sonner/style.css'

createApp(App).use(router).use(createPinia()).mount('#app')
