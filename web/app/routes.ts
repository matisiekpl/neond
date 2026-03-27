import { type RouteConfig, index, route } from "@react-router/dev/routes"

export default [
  index("routes/Home.tsx"),
  route("login", "routes/Login.tsx"),
  route("register", "routes/Register.tsx"),
  route("logout", "routes/Logout.tsx"),
  route(
    "dashboard",
    "routes/Dashboard.tsx",
    [
      index("routes/DashboardIndex.tsx"),
      route(
        "settings/organization",
        "routes/DashboardSettingsOrganization.tsx",
      ),
    ],
  ),
] satisfies RouteConfig
