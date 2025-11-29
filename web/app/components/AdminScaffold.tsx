import { NavLink, Outlet } from "react-router";

import "./AdminScaffold.css";
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
              <NavLink to="/admin/">확정 예약 목록</NavLink>
            </li>
            <li>
              <NavLink to="/admin/pending-bookings/">대기중 예약 목록</NavLink>
            </li>
            <li>
              <NavLink to="/admin/pending-refunds/">환불 요청 목록</NavLink>
            </li>
            <li>
              <NavLink to="/admin/reservations/">임의 예약</NavLink>
            </li>
            <li>
              <NavLink to="/admin/users/">고객 목록</NavLink>
            </li>
            <li>
              <NavLink to="/admin/groups/">그룹 목록</NavLink>
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
