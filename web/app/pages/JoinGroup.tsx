import { isAxiosError } from "axios";
import { useState, useEffect } from "react";
import { Link } from "react-router";

import "./JoinGroup.css";
import type { Route } from "./+types/JoinGroup";
import UserService from "../api/user";
import LogoType from "../assets/logotype.svg";
import { defaultErrorHandler, loaderErrorHandler } from "../lib/error";
import RequiresAuth, { type AuthProps } from "../lib/RequiresAuth";
import type { GroupWithUsers } from "../types/models/group";

export function meta(): Route.MetaDescriptors {
  return [{ title: "그룹 가입 | 드림하우스 합주실" }];
}

interface LoaderData {
  group: GroupWithUsers;
  redirectTo: string | null;
}

export async function clientLoader({
  params,
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.groupId) {
    throw new Error("groupId is not supplied");
  }

  const url = new URL(request.url);
  const searchParams = url.searchParams;
  const redirectTo = searchParams.get("redirect_to");

  try {
    const result = await UserService.getGroup(params.groupId);

    return { group: result.data.group, redirectTo };
  } catch (error) {
    throw loaderErrorHandler(error, request.url);
  }
}

export function JoinGroup({
  loaderData,
  auth,
}: Route.ComponentProps & AuthProps) {
  const group = loaderData.group;
  const [error, setError] = useState<string | null>(null);
  const [requestInProgress, setRequestInProgress] = useState(false);
  const [isDone, setDone] = useState(false);

  const redirectTo = loaderData.redirectTo ?? "/my/";

  useEffect(() => {
    if (
      group.users.find((v) => {
        return v.id === auth.user.id;
      }) !== undefined
    ) {
      setError(`이미 ${group.name} 그룹에 가입되어 있습니다.`);
    }
  }, [auth, group]);

  const joinGroup = async () => {
    setRequestInProgress(true);

    try {
      await UserService.joinGroup(group.id);
      setDone(true);
    } catch (error) {
      if (isAxiosError(error)) {
        const message = error.response?.data.message;
        if (message) {
          setError(message as string);
        }
      }
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  return (
    <div className="join-group">
      <Link to="/">
        <img className="logo" src={LogoType} alt="드림하우스 합주실" />
      </Link>
      {error ? (
        <p className="message">{error}</p>
      ) : (
        <>
          <p className="message">
            {!isDone && !error ? (
              <>
                <em>{group.name}</em>에 가입하시려면 다음 버튼을 눌러주세요.
              </>
            ) : (
              <>가입이 완료되었습니다.</>
            )}
          </p>
          {!isDone ? (
            <button
              onClick={joinGroup}
              className="cta"
              disabled={requestInProgress}
            >
              가입
            </button>
          ) : (
            <Link to={redirectTo} className="cta" replace>
              확인
            </Link>
          )}
        </>
      )}
    </div>
  );
}

export default RequiresAuth(JoinGroup);
