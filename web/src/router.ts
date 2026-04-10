import { createRouter, createWebHistory, type RouteLocationGeneric } from 'vue-router'
import LoginView from '@/views/Auth/LoginView.vue'
import RegisterView from '@/views/Auth/RegisterView.vue'
import LogoutView from '@/views/Auth/LogoutView.vue'
import DashboardLayout from '@/views/Dashboard/DashboardLayout.vue'
import ProjectsIndexView from '@/views/Dashboard/ProjectsIndexView.vue'
import ProjectView from '@/views/Dashboard/ProjectView.vue'
import ProjectSettingsView from '@/views/Dashboard/ProjectSettingsView.vue'
import SetupOrganizationView from '@/views/Dashboard/SetupOrganizationView.vue'
import OrganizationSettingsView from '@/views/Dashboard/OrganizationSettingsView.vue'
import DaemonView from '@/views/Dashboard/DaemonView.vue'

const routes = [
  { path: '/', redirect: '/login' },
  { path: '/login', component: LoginView, name: 'login' },
  { path: '/register', component: RegisterView, name: 'register' },
  { path: '/logout', component: LogoutView, name: 'logout' },
  { path: '/setup-organization', component: SetupOrganizationView, name: 'setup-organization' },
  {
    path: '/organizations/:organizationId',
    component: DashboardLayout,
    children: [
      {
        path: '',
        redirect: (to: RouteLocationGeneric) => ({
          name: 'projects.list',
          params: { organizationId: to.params['organizationId'] },
        }),
      },
      { path: 'projects', component: ProjectsIndexView, name: 'projects.list' },
      { path: 'projects/:projectId', component: ProjectView, name: 'projects.show' },
      { path: 'projects/:projectId/settings', component: ProjectSettingsView, name: 'projects.settings' },
      { path: 'settings/organization', component: OrganizationSettingsView, name: 'settings.organization' },
      { path: 'daemon', component: DaemonView, name: 'daemon' },
    ],
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})