import { AxiosError, isAxiosError } from "axios";
import { redirect } from "react-router";

interface Error {
  type: string;
  message: string;
}

function showErrorMessage(error: AxiosError<Error>) {
  if (error.response?.data?.message) {
    alert(error.response?.data?.message);
  } else {
    alert("일시적인 오류가 발생했습니다. 다시 시도해 주세요.");
  }
}

export function defaultErrorHandler(error: unknown) {
  if (isAxiosError(error)) {
    if (error.status !== 401) {
      showErrorMessage(error);
    }
  }
}

export function handleUnauthorizedError(error: unknown) {
  if (isAxiosError(error)) {
    if (error.status === 401) {
      throw redirect("/login");
    } else {
      showErrorMessage(error);
    }
  }
}
