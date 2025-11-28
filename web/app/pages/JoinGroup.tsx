import { useState, useEffect } from "react";
import { Link } from "react-router";

import "./JoinGroup.css";
import type { Route } from "./+types/JoinGroup";
import UserService from "../api/user";
import LogoType from "../assets/logotype.svg";
import { useAuth } from "../context/AuthContext";
import { defaultErrorHandler } from "../lib/error";
import RequiresAuth from "../lib/RequiresAuth";
import type { GroupId } from "../types/models/base";
import type { GroupWithUsers } from "../types/models/group";
import { isAxiosError } from "axios";

interface LoaderData {
  groupId: GroupId;
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.groupId) {
    throw new Error("groupId is not supplied");
  }
  return { groupId: params.groupId };
}

export function meta({}: Route.MetaArgs) {
  return [{ title: "그룹 가입 | 드림하우스 합주실" }];
}

export function JoinGroup({ loaderData }: { loaderData: LoaderData }) {
  const auth = useAuth();

  const [group, setGroup] = useState<GroupWithUsers | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [requestInProgress, setRequestInProgress] = useState(false);
  const [isDone, setDone] = useState(false);

  const fetchGroup = async () => {
    if (!auth || isDone) {
      return;
    }
    try {
      const result = await UserService.getGroup(loaderData.groupId);
      if (
        result.data.group.users.find((v) => {
          return v.id === auth?.user.id;
        }) !== undefined
      ) {
        setError(`이미 ${result.data.group.name} 그룹에 가입되어 있습니다.`);
      } else {
        setGroup(result.data.group);
      }
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  const joinGroup = async () => {
    setRequestInProgress(true);

    try {
      await UserService.joinGroup(loaderData.groupId);
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

  useEffect(() => {
    fetchGroup();
  }, [auth]);

  return (
    <div className="join-group">
      <Link to="/">
        <img className="logo" src={LogoType} alt="드림하우스 합주실" />
      </Link>
      {group ? (
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
            <Link to="/my" className="cta" replace>
              확인
            </Link>
          )}
        </>
      ) : null}
      {error ? <p className="message">{error}</p> : null}
    </div>
  );
}

export default RequiresAuth(JoinGroup);
