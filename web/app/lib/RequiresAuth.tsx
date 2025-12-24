import { useEffect } from "react";
import { useLocation, useNavigate } from "react-router";

import { useAuth, type AuthContextData } from "../context/AuthContext";
import { useEnv } from "../context/EnvContext";
import { isKakaoWebView, kakaoInAppLogin } from "./KakaoSDK";

export interface AuthProps {
  auth: AuthContextData;
}

export default function RequiresAuth<P extends object>(
  WrappedComponent: (
    props: P & AuthProps,
  ) => React.ReactElement | null | undefined,
  redirectPath?: string,
) {
  return function RequiresAuth(props: P) {
    const auth = useAuth();
    const env = useEnv();
    const location = useLocation();
    const navigate = useNavigate();

    useEffect(() => {
      if (auth === null) {
        const redirectTo = location.pathname + location.search;

        if (isKakaoWebView()) {
          kakaoInAppLogin(env, redirectTo);
        } else {
          const path =
            redirectPath ??
            `/login/?redirect_to=${encodeURIComponent(redirectTo)}`;
          navigate(path);
        }
      }
    }, [auth, env, location, navigate]);

    if (auth) {
      return <WrappedComponent auth={auth} {...props} />;
    }
  };
}
