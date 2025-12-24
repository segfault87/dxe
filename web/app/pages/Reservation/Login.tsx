import { useEffect } from "react";
import ReactGA from "react-ga4";
import { useNavigate } from "react-router";

import "./Login.css";
import type { Route } from "./+types/Login";
import KakaoLoginButton from "../../assets/kakao_login_large_wide.png";
import { useAuth } from "../../context/AuthContext";
import { useEnv } from "../../context/EnvContext";
import { kakaoLogin } from "../../lib/KakaoSDK";

export function meta(): Route.MetaDescriptors {
  return [{ title: "로그인 | 드림하우스 합주실" }];
}

export default function Login() {
  const env = useEnv();
  const auth = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    if (auth) {
      navigate("/reservation/");
    }
  }, [auth, navigate]);

  return (
    <div className="content-wrapper">
      <div className="login-panel">
        <p>
          예약을 진행하시기 위해서는 로그인이 필요합니다.
          <br />
          카카오 로그인을 통해 간편하게 가입하신 뒤에 이용하실 수 있습니다.
          <br />
          아래 버튼을 눌러 로그인해주세요.
        </p>
        <a
          className="kakao-login"
          onClick={() => {
            ReactGA.event("login_kakao", { from: "reservation" });
            kakaoLogin(env, "/reservation");
          }}
        >
          <img src={KakaoLoginButton} alt="카카오 로그인" />
        </a>
      </div>
      <div className="calendar">
        <h2>현재 예약 현황</h2>
        <iframe src="https://calendar.google.com/calendar/embed?src=c_c3419e5a4642a663fd9b60d901e46127c105c664e86fe90dbf43a4b20bbca8f3%40group.calendar.google.com&ctz=Asia%2FSeoul"></iframe>
      </div>
    </div>
  );
}
