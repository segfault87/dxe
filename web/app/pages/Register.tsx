import { useCallback, useState, type FormEvent } from "react";
import { useSearchParams } from "react-router";

import "./Register.css";
import type { Route } from "./+types/Register";
import AuthService from "../api/auth";
import checkPlateNumber from "../lib/PlateNumber";
import { defaultErrorHandler } from "../lib/error";

export function meta({}: Route.MetaArgs) {
  return [{ title: "등록 | 드림하우스 합주실" }];
}

export default function Register() {
  const [searchParams, _] = useSearchParams();
  const [name, setName] = useState(searchParams.get("name") ?? "");
  const [licensePlateNumber, setLicensePlateNumber] = useState("");
  const [isLoading, setLoading] = useState(false);

  const redirectTo = searchParams.get("redirect_to") ?? "/";

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
          document.location.href = redirectTo;
        } else {
          alert("등록에 실패했습니다.");
        }
      } catch (error) {
        defaultErrorHandler(error);
      } finally {
        setLoading(false);
      }
    },
    [name, licensePlateNumber, redirectTo, disabled],
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
