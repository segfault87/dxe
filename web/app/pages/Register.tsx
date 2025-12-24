import { useCallback, useEffect, useState, type FormEvent } from "react";
import ReactGA from "react-ga4";
import { useNavigate } from "react-router";

import "./Register.css";
import type { Route } from "./+types/Register";
import AuthService from "../api/auth";
import { useAuth, useAuthRefresh } from "../context/AuthContext";
import checkPlateNumber from "../lib/PlateNumber";
import { defaultErrorHandler } from "../lib/error";

export function meta({}: Route.MetaArgs) {
  return [{ title: "등록 | 드림하우스 합주실" }];
}

interface LoaderData {
  name: string;
  redirectTo: string;
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  const url = new URL(request.url);
  const searchParams = url.searchParams;

  const name = searchParams.get("name") ?? "";
  const redirectTo = searchParams.get("redirect_to") ?? "/";

  return { name, redirectTo };
}

export default function Register({ loaderData }: { loaderData: LoaderData }) {
  const auth = useAuth();
  const authRefresh = useAuthRefresh();
  const navigate = useNavigate();

  const [name, setName] = useState(loaderData.name);
  const [licensePlateNumber, setLicensePlateNumber] = useState("");
  const [isLoading, setLoading] = useState(false);

  const redirectTo = loaderData.redirectTo;

  useEffect(() => {
    if (auth) {
      navigate(redirectTo);
    }
  }, [auth, navigate, redirectTo]);

  useEffect(() => {
    ReactGA.event("sign_up");
  }, []);

  const disabled =
    isLoading ||
    name.trim().length == 0 ||
    checkPlateNumber(licensePlateNumber) === false;

  const handleSubmit = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();

      if (disabled) {
        return;
      }

      const formData = {
        name: name,
        licensePlateNumber: licensePlateNumber,
      };

      setLoading(true);

      try {
        const response = await AuthService.kakaoRegister(formData);

        if (response.status === 200) {
          ReactGA.event("signed_up");
          await authRefresh();
          navigate(redirectTo);
        } else {
          alert("등록에 실패했습니다.");
        }
      } catch (error) {
        defaultErrorHandler(error);
      } finally {
        setLoading(false);
      }
    },
    [authRefresh, name, licensePlateNumber, redirectTo, disabled, navigate],
  );

  return (
    <form onSubmit={handleSubmit} className="registration-form">
      <h2>고객 정보 입력</h2>
      <p>진행하시기 전에 아래의 정보만 기입해 주세요.</p>
      <div className="field">
        <label htmlFor="name">성함 (닉네임)</label>
        <input
          type="text"
          id="name"
          name="name"
          maxLength={40}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </div>
      <div className="field">
        <label htmlFor="license-plate-number">차량번호</label>
        <input
          type="text"
          id="license-plate-number"
          name="license-plate-number"
          maxLength={40}
          value={licensePlateNumber}
          placeholder="없을 시 생략가능"
          onChange={(e) => setLicensePlateNumber(e.target.value)}
        />
      </div>

      <button className="cta" type="submit" disabled={disabled}>
        확인
      </button>
    </form>
  );
}
