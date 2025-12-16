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
      <div className="footer">
        <span>상호명 : 디엑스이 스튜디오</span>
        <span>대표자명 : 박준규</span>
        <span>사업장 위치 : 서울특별시 강남구 삼성로 517 B101-2</span>
        <span>사업자등록번호 : 701-07-03619</span>
        <span>
          전화번호 : <a href="tel:+8250219445150">+82 0502-1944-5150</a>
        </span>
      </div>
    </main>
  );
}
