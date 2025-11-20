import { NavLink, Outlet } from "react-router";

import { useAuth } from "../context/AuthContext";

export default function AdminScaffold() {
  const auth = useAuth();

  if (!auth) {
    return null;
  } else if (!auth.user.isAdministrator) {
    return <main className="blocker">관리자만 접근 가능합니다.</main>;
  } else {
    return (
      <main>
        <div className="navigation">
          <ul>
            <li>
              <NavLink to="/admin/pending-bookings">대기중 예약 목록</NavLink>
            </li>
            <li>
              <NavLink to="/admin/reservations">임의 예약</NavLink>
            </li>
          </ul>
        </div>
        <div className="content">
          <Outlet />
        </div>
      </main>
    );
  }
}
