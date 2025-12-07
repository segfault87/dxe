export interface KakaoAuthRegisterRequest {
  name: string;
  licensePlateNumber?: string;
}

export interface HandleAuthRequest {
  handle: string;
  password: string;
}

export interface HandleAuthResponse {
  redirectTo: string;
}
