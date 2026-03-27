import {createRouter, createWebHistory} from 'vue-router'
import LoginView from '@/views/Auth/LoginView.vue'
import RegisterView from '@/views/Auth/RegisterView.vue'
import LogoutView from '@/views/Auth/LogoutView.vue'
import DashboardLayout from '@/layouts/DashboardLayout.vue'
import DashboardHomeView from '@/views/Dashboard/DashboardHomeView.vue'
import OrganizationSettingsView from '@/views/Dashboard/OrganizationSettingsView.vue'
import {ACCESS_TOKEN} from '@/stores/auth.store.ts'

const routes = [
  {path: '/login', component: LoginView, name: 'login', meta: {public: true}},
  {path: '/register', component: RegisterView, name: 'register', meta: {public: true}},
  {path: '/logout', component: LogoutView, name: 'logout', meta: {public: true}},
  {
    path: '/dashboard',
    component: DashboardLayout,
    meta: {requiresAuth: true},
    children: [
      {
        path: '',
        name: 'dashboard',
        component: DashboardHomeView,
        meta: {title: 'Dashboard'},
      },
      {
        path: 'settings/organization',
        name: 'org-settings',
        component: OrganizationSettingsView,
        meta: {title: 'Organization settings'},
      },
    ],
  },
  {path: '/', redirect: '/dashboard'},
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to) => {
  const token = localStorage.getItem(ACCESS_TOKEN)
  if (to.meta.requiresAuth && !token) {
    return {name: 'login', query: {return: to.fullPath}}
  }
  if (to.meta.public && token && (to.name === 'login' || to.name === 'register')) {
    return {name: 'dashboard'}
  }
  return true
})
