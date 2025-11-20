import { isAxiosError } from "axios";

export function defaultErrorHandler(error: unknown) {
  if (isAxiosError(error)) {
    if (error.response?.data?.message) {
      alert(error.response?.data?.message);
    }
  }
}
