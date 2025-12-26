import { NavLink, Outlet } from "react-router";

import "./AdminScaffold.css";
import type { Route } from "./+types/AdminScaffold";
import { useAuth } from "../context/AuthContext";

export function meta(): Route.MetaDescriptors {
  return [{ title: "드림하우스 합주실 관리자" }];
}

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
              <NavLink to="/admin/past-bookings/" end>
                과거 이용 내역
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/adhoc-reservations/" end>
                임의 예약
              </NavLink>
            </li>
            <li>
              <NavLink to="/admin/adhoc-parkings/" end>
                임의 주차
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
