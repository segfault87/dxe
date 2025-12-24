import { useLocation } from "react-router";

import { useAuth } from "../context/AuthContext";
import { useEnv } from "../context/EnvContext";
import { isKakaoWebView, kakaoInAppLogin } from "./KakaoSDK";

export default function RequiresAuth<P extends object>(
  WrappedComponent: (props: P) => React.ReactElement | null | undefined,
  redirectPath?: string,
) {
  return function RequiresAuth(props: P) {
    const auth = useAuth();
    const env = useEnv();
    const location = useLocation();

    if (auth === null) {
      let redirectTo = location.pathname;
      if (location.search) {
        redirectTo += location.search;
      }

      if (isKakaoWebView()) {
        kakaoInAppLogin(env, redirectTo);
      } else {
        const path =
          redirectPath ??
          `/login/?redirect_to=${encodeURIComponent(redirectTo)}`;
        window.location.href = path;
      }

      return <></>;
    } else {
      return <WrappedComponent {...props} />;
    }
  };
}
