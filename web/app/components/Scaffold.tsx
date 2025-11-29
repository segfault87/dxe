import { NavLink, Outlet } from "react-router";

import "./Scaffold.css";
import LogoType from "../assets/logotype.svg";
import { useAuth } from "../context/AuthContext";

export default function Scaffold() {
  const auth = useAuth();

  return (
    <main>
      <nav className="header">
        <div className="logotype">
          <NavLink to="/" end>
            <img src={LogoType} width="200" />
          </NavLink>
        </div>
        <div className="navigation">
          <NavLink to="/" end>
            소개
          </NavLink>
          <NavLink to="/guide/" end>
            이용 안내
          </NavLink>
          <NavLink to="/inquiries/" end>
            문의
          </NavLink>
          <NavLink to="/reservation/" end>
            예약하기
          </NavLink>
          <NavLink to="/my/" end>
            조회
          </NavLink>
          {auth?.user.isAdministrator ? (
            <NavLink to="/admin/" end>
              관리자
            </NavLink>
          ) : null}
        </div>
      </nav>
      <div className="content">
        <Outlet />
      </div>
    </main>
  );
}
