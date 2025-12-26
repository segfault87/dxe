import { useEffect } from "react";
import ReactGA from "react-ga4";
import { Link } from "react-router";

import type { Route } from "./+types/Error";
import SinglePage from "../components/SinglePage";

interface LoaderData {
  errorCategory: string | null;
  message: string | null;
  kakaoError: string | null;
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  const url = new URL(request.url);
  const searchParams = url.searchParams;

  return {
    errorCategory: searchParams.get("error_category"),
    message: searchParams.get("message"),
    kakaoError: searchParams.get("kakao_error"),
  };
}

export default function Error({ loaderData }: Route.ComponentProps) {
  useEffect(() => {
    ReactGA.event("error", {
      error_category: loaderData.errorCategory,
      message: loaderData.message,
      kakao_error: loaderData.kakaoError,
    });
  }, [loaderData]);

  const title =
    loaderData.errorCategory === "kakao_auth"
      ? "카카오 인증 중 문제가 발생했습니다."
      : "에러가 발생했습니다.";

  return (
    <SinglePage>
      <h2>{title}</h2>
      <p>{loaderData.message}</p>
      <div className="actions">
        <Link to="/" className="cta" replace>
          돌아가기
        </Link>
      </div>
    </SinglePage>
  );
}
