import {
  type RouteConfig,
  route,
  index,
  layout,
  prefix,
} from "@react-router/dev/routes";

export default [
  layout("./components/Scaffold.tsx", [
    index("./pages/Index.tsx"),
    route("guide", "./pages/Guide.tsx"),
    route("inquiries", "./pages/Inquiries.tsx"),
    route("reservation", "./pages/Reservation/Create.tsx"),
    route("reservation/:bookingId", "./pages/Reservation/Show.tsx"),
    route("login", "./pages/Login.tsx"),
    route("register", "./pages/Register.tsx"),
    route("my", "./pages/MyPage.tsx"),
    route("terms-of-service", "./pages/legal/TermsOfService.tsx"),
  ]),
  route("join/:groupId", "./pages/JoinGroup.tsx"),
  ...prefix("/admin", [
    layout("./components/AdminScaffold.tsx", [
      index("./pages/Admin/Index.tsx"),
      route("pending-bookings", "./pages/Admin/PendingBookings.tsx"),
      route("pending-refunds", "./pages/Admin/RefundPendingBookings.tsx"),
      route("groups", "./pages/Admin/Groups.tsx"),
      route("users", "./pages/Admin/Users.tsx"),
      route("reservations", "./pages/Admin/Reservations.tsx"),
    ]),
  ]),
] satisfies RouteConfig;
