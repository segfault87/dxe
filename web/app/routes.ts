import {
  type RouteConfig,
  route,
  index,
  layout,
} from "@react-router/dev/routes";

export default [
  layout("./components/Scaffold.tsx", [
    index("./pages/Index.tsx"),
    route("guide", "./pages/Guide.tsx"),
    route("inquiries", "./pages/Inquiries.tsx"),
    route("reservation", "./pages/Reservation.tsx"),
  ]),
] satisfies RouteConfig;
