import { Link } from "react-router";

import "./Recording.css";
import type { Route } from "./+types/Recording";
import BookingService from "../api/booking";
import LogoType from "../assets/logotype.svg";
import { defaultErrorHandler } from "../lib/error";
import RequiresAuth from "../lib/RequiresAuth";
import type { AudioRecording } from "../types/models/booking";

interface LoaderData {
  audioRecording: AudioRecording | null;
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.bookingId) {
    throw new Error("bookingId is not supplied");
  }

  try {
    const result = await BookingService.getAudioRecording(params.bookingId);
    return { audioRecording: result.data.audioRecording };
  } catch (error) {
    defaultErrorHandler(error);
    return { audioRecording: null };
  }
}

export function meta({}: Route.MetaArgs) {
  return [{ title: "레코딩 음원 다운로드 | 드림하우스 합주실" }];
}

export function Recording({ loaderData }: { loaderData: LoaderData }) {
  const { audioRecording } = loaderData;

  return (
    <div className="download-recording">
      <Link to="/">
        <img className="logo" src={LogoType} alt="드림하우스 합주실" />
      </Link>
      {audioRecording ? (
        <>
          <p className="message">
            음원 파일을 다운로드받으시려면 아래 버튼을 눌러주세요.
            <br />
            다운로드 기한:{" "}
            {audioRecording.expiresIn
              ? new Date(audioRecording.expiresIn).toLocaleString()
              : "-"}
          </p>
          <a href={audioRecording.url} className="cta">
            다운로드
          </a>
        </>
      ) : (
        <p className="message">파일이 없거나 다운로드 기한이 만료되었습니다.</p>
      )}
    </div>
  );
}

export default RequiresAuth(Recording);
