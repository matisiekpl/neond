import { type RouteConfig, index, route } from "@react-router/dev/routes"

export default [
  index("routes/auth/Home.tsx"),
  route("login", "routes/auth/Login.tsx"),
  route("register", "routes/auth/Register.tsx"),
  route("logout", "routes/auth/Logout.tsx"),
  route(
    "dashboard",
    "routes/dashboard/Dashboard.tsx",
    [
      index("routes/dashboard/DashboardIndex.tsx"),
      route("projects", "routes/projects/ProjectsIndex.tsx"),
      route("projects/:projectId", "routes/projects/ProjectView.tsx"),
      route("projects/:projectId/settings", "routes/projects/ProjectSettings.tsx"),
      route("setup-organization", "routes/organization/DashboardSetupOrganization.tsx"),
      route(
        "settings/organization",
        "routes/organization/DashboardSettingsOrganization.tsx",
      ),
    ],
  ),
] satisfies RouteConfig
