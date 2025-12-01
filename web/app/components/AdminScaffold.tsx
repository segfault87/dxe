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
              <NavLink to="/" end>
                처음으로{" "}
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/" end>
                확정 예약 목록
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/pending-bookings/" end>
                대기중 예약 목록
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/pending-refunds/" end>
                환불 요청 목록
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/adhoc-reservations/" end>
                임의 예약
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/users/" end>
                고객 목록
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/groups/" end>
                그룹 목록
              </NavLink>
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
