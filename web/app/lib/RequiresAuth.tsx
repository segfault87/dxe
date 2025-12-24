import { useEffect } from "react";
import { useLocation } from "react-router";

import { useAuth, type AuthContextData } from "../context/AuthContext";
import { ErrorObject } from "../lib/error";

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
    const location = useLocation();

    useEffect(() => {
      if (auth === null) {
        const redirectTo = redirectPath ?? location.pathname + location.search;
        throw new ErrorObject({ type: "unauthorized", redirectTo: redirectTo });
      }
    }, [auth, location]);
    if (auth) {
      return <WrappedComponent auth={auth} {...props} />;
    }
  };
}
