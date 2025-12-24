import { AxiosError, isAxiosError } from "axios";
import { data, redirect } from "react-router";
import { isKakaoWebView, kakaoInAppLogin } from "./KakaoSDK";

interface Error {
  type: string;
  message: string;
}

function getErrorMessage(error: AxiosError<Error>) {
  if (error.response?.data?.message) {
    return error.response?.data?.message ?? error.message;
  } else {
    return "일시적인 오류가 발생했습니다. 다시 시도해 주세요.";
  }
}

export function defaultErrorHandler(error: unknown) {
  if (isAxiosError(error)) {
    if (error.status !== 401) {
      alert(getErrorMessage(error));
    }
  }
}

export function loaderErrorHandler(error: unknown, url: string) {
  if (isAxiosError(error)) {
    if (error.status === 401) {
      const encodedUrl = new URL(url);
      const redirectTo = encodedUrl.pathname + encodedUrl.search;
      if (isKakaoWebView()) {
        kakaoInAppLogin(import.meta.env.VITE_URL_BASE, redirectTo);
        return data(null, { status: 302 });
      } else {
        return redirect(`/login?redirect_to=${encodeURIComponent(redirectTo)}`);
      }
    }
    return data(getErrorMessage(error));
  } else {
    return data("일시적인 오류가 발생했습니다. 다시 시도해 주세요.");
  }
}

export function handleUnauthorizedError(error: unknown) {
  if (isAxiosError(error)) {
    if (error.status === 401) {
      throw redirect("/login");
    } else {
      alert(getErrorMessage(error));
    }
  }
}
