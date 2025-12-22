import type { Route } from "./+types/BookingDetails";
import AdminService from "../../api/admin";
import Section from "../../components/Section";
import type {
  AudioRecording,
  BookingWithPayments,
  TelemetryEntry,
} from "../../types/models/booking";

interface LoaderData {
  booking: BookingWithPayments;
  telemetryEntries: TelemetryEntry[];
  audioRecording: AudioRecording | null;
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.bookingId) {
    throw new Error("bookingId is not supplied");
  }

  const result = await AdminService.getBooking(params.bookingId);

  return {
    booking: result.data.booking,
    telemetryEntries: result.data.telemetryEntries,
    audioRecording: result.data.audioRecording,
  };
}

export default function BookingDetails({ loaderData }: Route.ComponentProps) {
  const { booking, telemetryEntries, audioRecording } = loaderData;

  return (
    <>
      <Section id="telemetry" title="예약 정보">
        <ul>
          <li>ID: {booking.booking.id}</li>
          <li>
            예약시간: {new Date(booking.booking.bookingStart).toLocaleString()}{" "}
            - {new Date(booking.booking.bookingEnd).toLocaleString()}
          </li>
          <li>예약자 : {booking.booking.holder.name}</li>
          <li>고객명 : {booking.booking.customer.name}</li>
          <li>상태 : {booking.booking.status}</li>
        </ul>
      </Section>
      <Section id="telemetry" title="측정 데이터">
        {telemetryEntries.map((v) => (
          <img
            className="telemetry-plot"
            src={`/api/admin/booking/${booking.booking.id}/telemetry?type=${v.type}`}
            key={v.type}
            alt={v.type}
          />
        ))}
      </Section>
      {audioRecording ? (
        <Section id="recirdubg" title="측정 데이터">
          <a href={audioRecording.url}>다운로드</a>
        </Section>
      ) : null}
    </>
  );
}
