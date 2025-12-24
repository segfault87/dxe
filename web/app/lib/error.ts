import { AxiosError, isAxiosError } from "axios";
import { redirect } from "react-router";

interface RemoteError {
  type: "remote";
  remoteType: string;
  message: string;
}

interface AuthError {
  type: "unauthorized";
  redirectTo: string;
}

interface UnknownError {
  type: "unknown";
  error?: Error;
}

type ErrorPayload = RemoteError | AuthError | UnknownError;

export class ErrorObject extends Error {
  data: ErrorPayload;

  constructor(data: ErrorPayload) {
    super();
    this.data = data;
  }
}

function getErrorMessage(error: AxiosError<RemoteError>) {
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

export function loaderErrorHandler(error: unknown, urlString: string) {
  if (isAxiosError(error)) {
    if (error.status === 401) {
      const url = new URL(urlString);
      return new ErrorObject({
        type: "unauthorized",
        redirectTo: url.pathname + url.search,
      });
    } else if (error.response?.data?.type && error.response?.data?.message) {
      return new ErrorObject({
        type: "remote",
        remoteType: error.response?.data?.type,
        message: error.response?.data?.message,
      });
    }
  } else if (error instanceof Error) {
    return new ErrorObject({
      type: "unknown",
      error: error,
    });
  } else {
    return new ErrorObject({
      type: "unknown",
    });
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
