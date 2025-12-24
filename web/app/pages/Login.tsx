import { useEffect } from "react";
import ReactGA from "react-ga4";
import { useAuth } from "../context/AuthContext";
import { useNavigate, useSearchParams } from "react-router";

import "./Login.css";
import type { Route } from "./+types/Login";
import KakaoLoginButton from "../assets/kakao_login_large_wide.png";
import { useEnv } from "../context/EnvContext";
import { kakaoLogin } from "../lib/KakaoSDK";

export function meta({}: Route.MetaArgs) {
  return [{ title: "로그인 | 드림하우스 합주실" }];
}

export default function Login() {
  const env = useEnv();
  const auth = useAuth();
  const navigate = useNavigate();
  const [searchParams, _] = useSearchParams();
  const redirectTo: string | null = searchParams.get("redirect_to");

  useEffect(() => {
    if (auth) {
      navigate(redirectTo ?? "/");
    }
  }, [auth, navigate, redirectTo]);

  const quotedRedirectTo = redirectTo ? encodeURI(redirectTo) : null;

  return (
    <div className="login-panel">
      <p>
        드림하우스 합주실이 제공하는 서비스를 이용하려면 로그인이 필요합니다.
        <br />
        카카오 로그인을 통해 간편하게 가입하신 뒤에 이용하실 수 있습니다.
        <br />
        아래 버튼을 눌러 로그인해주세요.
      </p>
      <a
        className="kakao-login"
        onClick={() => {
          ReactGA.event("login_kakao", { from: "else" });
          kakaoLogin(env, quotedRedirectTo);
        }}
      >
        <img src={KakaoLoginButton} alt="카카오 로그인" />
      </a>
    </div>
  );
}
