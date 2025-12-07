import API from "../api";
import type {
  HandleAuthRequest,
  HandleAuthResponse,
  KakaoAuthRegisterRequest,
} from "../types/handlers/auth";

const kakaoRegister = (data: KakaoAuthRegisterRequest) => {
  return API.post("/auth/kakao", data);
};

const handleAuth = (redirectTo: string | null, data: HandleAuthRequest) => {
  if (redirectTo) {
    return API.post<HandleAuthResponse>(
      `/auth/login?redirect_to=${encodeURI(redirectTo)}`,
      data,
    );
  } else {
    return API.post<HandleAuthResponse>("/auth/login", data);
  }
};

const AuthService = {
  kakaoRegister,
  handleAuth,
};

export default AuthService;
