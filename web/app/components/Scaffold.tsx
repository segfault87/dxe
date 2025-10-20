import { NavLink, Outlet } from "react-router";

import "./Scaffold.css";
import LogoType from "../assets/logotype.svg";

export default function Scaffold() {
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
          <NavLink to="/guide" end>
            이용 안내
          </NavLink>
          <NavLink to="/inquiries" end>
            문의
          </NavLink>
        </div>
      </nav>
      <div className="content">
        <Outlet />
      </div>
    </main>
  );
}
