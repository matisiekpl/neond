import {createRouter, createWebHistory} from 'vue-router'
import LoginView from '@/views/Auth/LoginView.vue'
import RegisterView from '@/views/Auth/RegisterView.vue'
import LogoutView from '@/views/Auth/LogoutView.vue'

const routes = [
  {path: '/login', component: LoginView, name: 'login'},
  {path: '/register', component: RegisterView, name: 'register'},
  {path: '/logout', component: LogoutView, name: 'logout'},
  {path: '/', redirect: '/login'},
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
