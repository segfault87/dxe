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
    route("guide/", "./pages/Guide.tsx"),
    route("inquiries/", "./pages/Inquiries.tsx"),
    route("reservation/", "./pages/Reservation/Create.tsx"),
    route("reservation/login/", "./pages/Reservation/Login.tsx"),
    route(
      "reservation/payment/toss/success/",
      "./pages/Reservation/Payment/TossPaymentSuccess.tsx",
    ),
    route(
      "reservation/payment/toss/fail/",
      "./pages/Reservation/Payment/TossPaymentFail.tsx",
    ),
    route("reservation/:bookingId", "./pages/Reservation/Show.tsx"),
    route("reservation/:bookingId/amend", "./pages/Reservation/Amend.tsx"),
    route("login/", "./pages/Login.tsx"),
    route("temp-login/", "./pages/HandleLogin.tsx"),
    route("register/", "./pages/Register.tsx"),
    route("my/", "./pages/MyPage.tsx"),
    route("terms-of-service/", "./pages/legal/TermsOfService.tsx"),
  ]),
  route("booking/:bookingId/recording/", "./pages/Recording.tsx"),
  route("join/:groupId", "./pages/JoinGroup.tsx"),
  ...prefix("/admin", [
    layout("./components/AdminScaffold.tsx", [
      index("./pages/Admin/ConfirmedBookings.tsx"),
      route("pending-bookings/", "./pages/Admin/PendingBookings.tsx"),
      route("pending-refunds/", "./pages/Admin/RefundPendingBookings.tsx"),
      route("groups/", "./pages/Admin/Groups.tsx"),
      route("users/", "./pages/Admin/Users.tsx"),
      route("adhoc-reservations/", "./pages/Admin/AdhocReservations.tsx"),
      route("adhoc-parkings/", "./pages/Admin/AdhocParkings.tsx"),
    ]),
  ]),
] satisfies RouteConfig;
