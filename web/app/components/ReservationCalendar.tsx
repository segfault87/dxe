import "./ReservationCalendar.css";
import { useEnv } from "../context/EnvContext";

export default function ReservationCalendar() {
  const env = useEnv();

  return (
    <iframe
      className="reservation-calendar"
      src={`https://calendar.google.com/calendar/embed?src=${env.googleCalendarId}&ctz=Asia%2FSeoul`}
    />
  );
}
