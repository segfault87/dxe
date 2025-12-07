import { useState } from "react";
import { useSearchParams } from "react-router";

import "./HandleLogin.css";
import type { Route } from "./+types/HandleLogin";
import AuthService from "../api/auth";
import { defaultErrorHandler } from "../lib/error";
import { useNavigate } from "react-router";

export function meta({}: Route.MetaArgs) {
  return [{ title: "로그인 | 드림하우스 합주실" }];
}

export default function Login() {
  const navigate = useNavigate();

  const [handle, setHandle] = useState("");
  const [password, setPassword] = useState("");
  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const [searchParams, _] = useSearchParams();
  const redirectTo: string | null = searchParams.get("redirect_to");

  const handleLogin = async () => {
    if (!handle || !password) {
      return;
    }

    setRequestInProgress(true);

    try {
      const result = await AuthService.handleAuth(redirectTo, {
        handle: handle,
        password: password,
      });

      navigate(result.data.redirectTo);
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  return (
    <div className="login-panel">
      <p>PG 검수를 위한 임시 로그인 페이지입니다.</p>
      <div className="cred-login">
        <div className="row">
          <label>아이디</label>
          <input
            type="text"
            value={handle}
            onChange={(e) => setHandle(e.target.value)}
          />
        </div>
        <div className="row">
          <label>비밀번호</label>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        <button
          disabled={!handle || !password || isRequestInProgress}
          onClick={handleLogin}
          className="cta"
        >
          로그인
        </button>
      </div>
    </div>
  );
}
