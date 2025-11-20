import ReactCalendar from "react-calendar";
import "react-calendar/dist/Calendar.css";

interface CalendarProps {
  start: Date;
  end: Date;
  onSelect: (date: Date | null) => void;
}

export default function Calendar(props: CalendarProps) {
  return (
    <ReactCalendar
      minDate={props.start}
      maxDate={props.end}
      onChange={(value) => {
        props.onSelect(value as Date | null);
      }}
    />
  );
}
