import { createRouter, createWebHistory, type RouteLocationGeneric } from 'vue-router'
import LoginView from '@/views/Auth/LoginView.vue'
import RegisterView from '@/views/Auth/RegisterView.vue'
import LogoutView from '@/views/Auth/LogoutView.vue'
import DashboardLayout from '@/views/Dashboard/DashboardLayout.vue'
import ProjectsIndexView from '@/views/Dashboard/Project/ProjectsIndexView.vue'
import ProjectView from '@/views/Dashboard/Project/ProjectView.vue'
import ProjectSettingsView from '@/views/Dashboard/Project/ProjectSettingsView.vue'
import SetupOrganizationView from '@/views/Dashboard/Organization/SetupOrganizationView.vue'
import OrganizationSettingsView from '@/views/Dashboard/Organization/OrganizationSettingsView.vue'
import DaemonView from '@/views/Dashboard/Daemon/DaemonView.vue'
import DaemonMetricsView from '@/views/Dashboard/Daemon/DaemonMetricsView.vue'
import DaemonLogsView from '@/views/Dashboard/Daemon/DaemonLogsView.vue'
import UsersView from '@/views/Dashboard/Users/UsersView.vue'
import BranchView from '@/views/Dashboard/Branch/BranchView.vue'
import BranchDataView from '@/views/Dashboard/Branch/BranchDataView.vue'
import BranchSqlView from '@/views/Dashboard/Branch/BranchSqlView.vue'
import BranchMetricsView from '@/views/Dashboard/Branch/BranchMetricsView.vue'
import BranchLogsView from '@/views/Dashboard/Branch/BranchLogsView.vue'

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
      {
        path: 'projects/:projectId/branches/:branchId',
        component: BranchView,
        children: [
          {
            path: '',
            redirect: (to: RouteLocationGeneric) => ({
              name: 'projects.branches.data',
              params: {
                organizationId: to.params['organizationId'],
                projectId: to.params['projectId'],
                branchId: to.params['branchId'],
              },
            }),
          },
          { path: 'data/:table?', component: BranchDataView, name: 'projects.branches.data' },
          { path: 'sql', component: BranchSqlView, name: 'projects.branches.sql' },
          { path: 'metrics', component: BranchMetricsView, name: 'projects.branches.metrics' },
          { path: 'logs/:component', component: BranchLogsView, name: 'projects.branches.logs' },
        ],
      },
      { path: 'settings/organization', component: OrganizationSettingsView, name: 'settings.organization' },
      { path: 'daemon', component: DaemonView, name: 'daemon' },
      { path: 'daemon/monitoring', component: DaemonMetricsView, name: 'daemon.monitoring' },
      { path: 'daemon/logs/:component', component: DaemonLogsView, name: 'daemon.logs' },
      { path: 'users', component: UsersView, name: 'users' },
    ],
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})