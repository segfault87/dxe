import API from "../api";
import type { KakaoAuthRegisterRequest } from "../types/handlers/auth";

const kakaoRegister = (data: KakaoAuthRegisterRequest) => {
  return API.post("/auth/kakao", data);
};

const AuthService = {
  kakaoRegister,
};

export default AuthService;
