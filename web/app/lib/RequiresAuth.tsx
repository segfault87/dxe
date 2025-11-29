import { useLocation } from "react-router";

import { useAuth } from "../context/AuthContext";

export default function RequiresAuth<P extends object>(
  WrappedComponent: (props: P) => React.ReactElement | null | undefined,
  redirectPath?: string,
) {
  return function RequiresAuth(props: P) {
    const auth = useAuth();
    const location = useLocation();

    if (auth === null) {
      let redirectTo = location.pathname;
      if (location.search) {
        redirectTo += location.search;
      }

      const path = redirectPath ?? `/login/?redirect_to=${redirectTo}`;
      window.location.href = path;

      return <></>;
    } else {
      return <WrappedComponent {...props} />;
    }
  };
}
